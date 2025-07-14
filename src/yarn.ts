// Copied from https://github.com/yarnpkg/berry/blob/9155359cb633748cd806117717e5709ad2869791/packages/yarnpkg-core/sources

import { BinaryLike, createHash } from 'node:crypto'
import querystring from 'node:querystring'
import semver from 'semver'

/**
 * Unique hash of a package descriptor. Used as key in various places so that
 * two descriptors can be quickly compared.
 */
export type IdentHash = string

/**
 * Combination of a scope and name, bound with a hash suitable for comparisons.
 *
 * Use `parseIdent` to turn ident strings (`@types/node`) into the ident
 * structure ({scope: `types`, name: `node`}), `makeIdent` to create a new one
 * from known parameters, or `stringifyIdent` to retrieve the string as you'd
 * see it in the `dependencies` field.
 */
export interface Ident {
  /**
   * Unique hash of a package scope and name. Used as key in various places,
   * so that two idents can be quickly compared.
   */
  identHash: IdentHash

  /**
   * Scope of the package, without the `@` prefix (eg. `types`).
   */
  scope: string | null

  /**
   * Name of the package (eg. `node`).
   */
  name: string
}

/**
 * Unique hash of a package descriptor. Used as key in various places so that
 * two descriptors can be quickly compared.
 */
export type DescriptorHash = string

/**
 * Descriptors are just like idents (including their `identHash`), except that
 * they also contain a range and an additional comparator hash.
 *
 * Use `parseRange` to turn a descriptor string into this data structure,
 * `makeDescriptor` to create a new one from an ident and a range, or
 * `stringifyDescriptor` to generate a string representation of it.
 */
export interface Descriptor extends Ident {
  /**
   * Unique hash of a package descriptor. Used as key in various places, so
   * that two descriptors can be quickly compared.
   */
  descriptorHash: DescriptorHash

  /**
   * The range associated with this descriptor. (eg. `^1.0.0`)
   */
  range: string
}

/**
 * Unique hash of a package locator. Used as key in various places so that
 * two locators can be quickly compared.
 */
export type LocatorHash = string

/**
 * Locator are just like idents (including their `identHash`), except that
 * they also contain a reference and an additional comparator hash. They are
 * in this regard very similar to descriptors except that each descriptor may
 * reference multiple valid candidate packages whereas each locators can only
 * reference a single package.
 *
 * This interesting property means that each locator can be safely turned into
 * a descriptor (using `convertLocatorToDescriptor`), but not the other way
 * around (except in very specific cases).
 */
export interface Locator extends Ident {
  /**
   * Unique hash of a package locator. Used as key in various places so that
   * two locators can be quickly compared.
   */
  locatorHash: LocatorHash

  /**
   * A package reference uniquely identifies a package (eg. `1.2.3`).
   */
  reference: string
}

export function makeHash(...args: (BinaryLike | null)[]) {
  const hash = createHash(`sha512`)

  let acc = ``
  for (const arg of args) {
    if (typeof arg === `string`) {
      acc += arg
    } else if (arg) {
      if (acc) {
        hash.update(acc)
        acc = ``
      }

      hash.update(arg)
    }
  }

  if (acc) { hash.update(acc) }

  return hash.digest(`hex`)
}

/**
 * Creates a package ident.
 *
 * @param scope The package scope without the `@` prefix (eg. `types`)
 * @param name The name of the package
 */
export function makeIdent(scope: string | null, name: string): Ident {
  if (scope?.startsWith(`@`)) { throw new Error(`Invalid scope: don't prefix it with '@'`) }

  return { identHash: makeHash(scope, name), scope, name }
}

/**
 * Creates a package locator.
 *
 * @param ident The base ident (see `makeIdent`)
 * @param reference The reference to attach (eg. `1.0.0`)
 */
export function makeLocator(ident: Ident, reference: string): Locator {
  return { identHash: ident.identHash, scope: ident.scope, name: ident.name, locatorHash: makeHash(ident.identHash, reference), reference }
}

const LOCATOR_REGEX_STRICT = /^(?:@([^/]+?)\/)?([^@/]+?)(?:@(.+))$/
const LOCATOR_REGEX_LOOSE = /^(?:@([^/]+?)\/)?([^@/]+?)(?:@(.+))?$/

/**
 * Parses a `string` into a locator
 *
 * Returns `null` if the locator cannot be parsed.
 *
 * @param string The locator string (eg. `lodash@1.0.0`)
 * @param strict If `false`, the reference is optional (`unknown` will be used as fallback)
 */
export function tryParseLocator(string: string, strict: boolean = false): Locator | null {
  const match = strict
    ? string.match(LOCATOR_REGEX_STRICT)
    : string.match(LOCATOR_REGEX_LOOSE)

  if (!match) { return null }

  const [, scope, name, reference] = match
  if (reference === `unknown`) { throw new Error(`Invalid reference (${string})`) }

  const realScope = typeof scope !== `undefined`
    ? scope
    : null

  const realReference = typeof reference !== `undefined`
    ? reference
    : `unknown`

  return makeLocator(makeIdent(realScope, name), realReference)
}

/**
 * Returns a string from an ident, formatted as a slug (eg. `@types-lodash`).
 */
export function slugifyIdent(ident: Ident) {
  if (ident.scope !== null) {
    return `@${ident.scope}-${ident.name}`
  } else {
    return ident.name
  }
}

const TRAILING_COLON_REGEX = /:$/

/**
 * Returns a string from a locator, formatted as a slug (eg. `@types-lodash-npm-1.0.0-abcdef1234`).
 */
export function slugifyLocator(locator: Locator) {
  const { protocol, selector } = parseRange(locator.reference)

  const humanProtocol = protocol !== null
    ? protocol.replace(TRAILING_COLON_REGEX, ``)
    : `exotic`

  const humanVersion = semver.valid(selector)

  const humanReference = humanVersion !== null
    ? `${humanProtocol}-${humanVersion}`
    : `${humanProtocol}`

  // 10 hex characters means that 47 different entries have 10^-9 chances of
  // causing a hash collision. Since this hash is joined with the package name
  // (making it highly unlikely you'll have more than a handful of instances
  // of any single package), this should provide a good enough guard in most
  // cases.
  //
  // Also note that eCryptfs eats some bytes, so the theoretical maximum for a
  // file size is around 140 bytes (but we don't need as much, as explained).
  const hashTruncate = 10

  return `${slugifyIdent(locator)}-${humanReference}-${locator.locatorHash.slice(0, hashTruncate)}`
}

type ParseRangeOptions = {
  /** Throw an error if bindings are missing */
  requireBindings?: boolean
  /** Throw an error if the protocol is missing or is not the specified one */
  requireProtocol?: boolean | string
  /** Throw an error if the source is missing */
  requireSource?: boolean
  /** Whether to parse the selector as a query string */
  parseSelector?: boolean
}

type ParseRangeReturnType<Opts extends ParseRangeOptions> =
  & ({params: Opts extends {requireBindings: true} ? querystring.ParsedUrlQuery : querystring.ParsedUrlQuery | null})
  & ({protocol: Opts extends {requireProtocol: true | string} ? string : string | null})
  & ({source: Opts extends {requireSource: true} ? string : string | null})
  & ({selector: Opts extends {parseSelector: true} ? querystring.ParsedUrlQuery : string})

const RANGE_REGEX = /^([^#:]*:)?((?:(?!::)[^#])*)(?:#((?:(?!::).)*))?(?:::(.*))?$/

/**
 * Parses a range into its constituents. Ranges typically follow these forms,
 * with both `protocol` and `bindings` being optionals:
 *
 *     <protocol>:<selector>::<bindings>
 *     <protocol>:<source>#<selector>::<bindings>
 *
 * The selector is intended to "refine" the source, and is required. The source
 * itself is optional (for instance we don't need it for npm packages, but we
 * do for git dependencies).
 */
export function parseRange<Opts extends ParseRangeOptions>(range: string, opts?: Opts): ParseRangeReturnType<Opts> {
  const match = range.match(RANGE_REGEX)
  if (match === null) { throw new Error(`Invalid range (${range})`) }

  const protocol = typeof match[1] !== `undefined`
    ? match[1]
    : null

  if (typeof opts?.requireProtocol === `string` && protocol !== opts!.requireProtocol) {
    throw new Error(`Invalid protocol (${protocol})`)
  } else if (opts?.requireProtocol && protocol === null) {
    throw new Error(`Missing protocol (${protocol})`)
  }

  const source = typeof match[3] !== `undefined`
    ? decodeURIComponent(match[2])
    : null

  if (opts?.requireSource && source === null) { throw new Error(`Missing source (${range})`) }

  const rawSelector = typeof match[3] !== `undefined`
    ? decodeURIComponent(match[3])
    : decodeURIComponent(match[2])

  const selector = (opts?.parseSelector)
    ? querystring.parse(rawSelector)
    : rawSelector

  const params = typeof match[4] !== `undefined`
    ? querystring.parse(match[4])
    : null

  // @ts-expect-error
  return { protocol, source, selector, params }
}

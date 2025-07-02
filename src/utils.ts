import k from 'kleur'

export function info(message: string): undefined {
  console.log(k.bold(k.bgBlue(' INFO ') + ' ' + k.white(message)))
}

export function error(message: string, error?: any): never {
  console.log(k.bold(k.bgRed(' ERROR ') + ' ' + k.white(message)))
  if (error) console.error(error)
  process.exit(1)
}

export function success(message: string): never {
  console.log(k.bold(k.bgGreen(' SUCCESS ') + ' ' + k.white(message)))
  process.exit(0)
}

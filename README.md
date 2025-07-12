# Cirno

Zero-Install Package Manager for Node.js.

## Usage

### `cirno init`

- `-f, --force`: overwrite existing environment.

Initialize a new Cirno environment.

### `cirno import <src>`

Import an application from a local path or URL.

Arguments after `--` will be passed to `yarn`.

### `cirno export <id> <dest>`

Export an application (or backup) to a local path.

### `cirno clone <id>`

Clone an application (or backup).

### `cirno remove <id>`

Remove an application (or backup).

### `cirno backup <id>`

Backup an application. See [Backup Timeline](#backup-timeline) for more information.

### `cirno restore <id>`

Restore to a backup. See [Backup Timeline](#backup-timeline) for more information.

### `cirno list`

- `--json`: output as JSON.

List all applications in the environment.

### `cirno yarn <id>`

Execute `yarn` in an application.

Arguments after `--` will be passed to `yarn`.

### `cirno gc`

Remove unused packages from the cache.

## Concepts

### Application

An application is a Node.js workspace. It may contain sub-workspaces, but they all share the same lockfile.

Applications can be imported via `cirno import` and exported via `cirno export`. They can also be cloned and removed.

### Bundle

A bundle is a zip file containing a zero-install application.

`cirno export` will pack all the dependencies and the package manager of an application so that installation requires no network connection.

### Shared Cache

Every bundle supports zero-install, which means that `import`-ing a bundle needs no extra network requests other than downloading the bundle itself.

On the other hand, local applications may have many duplicated dependencies. To reduce the disk usage, Cirno manages a shared cache for all applications.

When you `import` an application, Cirno will move all the dependencies to the shared cache. When you `export` an application, Cirno will copy the dependencies from the shared cache to the bundle.

Finally, Cirno support garbage collection for the shared cache. You can use `cirno gc` to remove unused packages from the cache. This will allow Cirno to have even less disk usage than Yarn or pnpm stores.

### Backup Timeline

Cirno supports backup and restore. You can use `cirno backup` to create a backup of an application, and use `cirno restore` to restore an application to a backup.

For example:

```sh
$ cirno backup A
> Successfully created a backup instance B.
$ cirno backup A
> Successfully created a backup instance C.
$ cirno backup A
> Successfully created a backup instance D.
```

This will create four instances with the following timeline:

```
B -> C -> D -> A
```

Instance A is called a **head instance**, while instances B, C, D are called the **base instances**. These two types of instances have different behaviors:

- You can only `backup` a head instance. For example, the following command will fail:

  ```sh
  $ cirno backup D
  > Cannot backup a base instance.
  ```

  Despite this, you can still `clone` a backup instance.

- You can only `restore` to backup instance. For example, the following command will fail:

  ```sh
  $ cirno restore A
  > Cannot restore to a head instance.
  ```

- When you `restore` to a backup instance, all the subsequent instances will be removed. For example, if you `restore` to instance C, the timeline will be:

  ```
  B -> C
  ```

  In this case, C will become the head instance on which you can make further backups.

- When you `remove` an instance, the next instance in the timeline will be linked to the previous instance. For example, if you `remove` instance C, the timeline will be:

  ```
  B -> D -> A
  ```

  In particular, if you `remove` the head instance, the last backup instance will become the head instance.

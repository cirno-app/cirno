# Cirno

Zero-Install Package Manager for Node.js.

## Usage

### `cirno init`

- `-f, --force`: Overwrite existing project.

Initialize a new Cirno project.

### `cirno import <src>`

Import an instance from a local path or URL.

Arguments after `--` will be passed to `yarn`.

### `cirno export <id> <dest>`

Export an instance to a local path.

### `cirno clone <id>`

Clone an instance.

### `cirno backup <id>`

Backup an instance. See [Backup Timeline](#backup-timeline) for more information.

### `cirno restore <id>`

Restore to a backup instance. See [Backup Timeline](#backup-timeline) for more information.

### `cirno remove <id>`

Remove an instance.

### `cirno list`

- `--json`: Output as JSON.

List all instances in the project.

### `cirno show <id>`

Show the information of an instance.

### `cirno yarn <id>`

Execute `yarn` in an instance.

Arguments after `--` will be passed to `yarn`.

### `cirno gc`

Remove unused packages from the cache.

## Concepts

### Instance

### Bundle

### Shared Cache

Every bundled instance supports zero-install, which means that `import`-ing an instance needs no extra network requests other than downloading the instance itself.

On the other hand, local instances may have many duplicated dependencies. To reduce the disk usage, Cirno manages a shared cache for all instances.

When you `import` an instance, Cirno will move all the dependencies to the shared cache. When you `export` an instance, Cirno will copy the dependencies from the shared cache to the instance.

Also, Cirno support garbage collection for the shared cache. You can use `cirno gc` to remove unused packages from the cache. This will allow Cirno to have even less disk usage than Yarn or pnpm stores.

### Backup Timeline

Cirno supports backup and restore. You can use `cirno backup` to create a backup of an instance, and use `cirno restore` to restore to a backup instance.

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

Workspace A is called the *head* instance, and instances B, C, D are called *base* instances. These two types of instances have different behaviors:

- You cannot `backup` a base instance. For example, the following command will fail:

  ```sh
  $ cirno backup D
  > Cannot backup a base instance.
  ```

  Despite this, you can still `clone` a base instance.

- You cannot `restore` to a head instance. For example, the following command will fail:

  ```sh
  $ cirno restore A
  > Cannot restore to a head instance.
  ```

- When you `restore` to a base instance, all the following instances will be removed. For example, if you `restore` to instance C, the timeline will be:

  ```
  B -> C
  ```

  In this case, C will become the head instance on which you can make further backups.

- When you `remove` a base instance, the next node in the timeline will be linked to the previous node. For example, if you `remove` instance C, the timeline will be:

  ```
  B -> D -> A
  ```

- When you `remove` a head instance, all base instances will be removed as well.

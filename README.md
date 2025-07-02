# Cirno

Zero-Install Package Manager for Node.js.

## Features

- Zero-Install
- Shared Cache with GC

## Usage

### `cirno init`

- `-f, --force`: Overwrite existing project.

Initialize a new Cirno project.

### `cirno import <src>`

Import a workspace from a local path or URL.

Arguments after `--` will be passed to `yarn`.

### `cirno export <id> <dest>`

Export a workspace to a local path.

### `cirno clone <id>`

Clone a workspace.

### `cirno backup <id>`

Backup a workspace. See [Backup](#backup) for more information.

### `cirno restore <id>`

Restore to a backup workspace. See [Backup](#backup) for more information.

### `cirno remove <id>`

Remove a workspace.

### `cirno list`

- `--json`: Output as JSON.

List all workspaces in the project.

### `cirno show <id>`

Show the information of a workspace.

### `cirno yarn <id>`

Execute `yarn` in a workspace.

Arguments after `--` will be passed to `yarn`.

### `cirno gc`

Remove unused packages from the cache.

## Backup

Cirno supports backup and restore. You can use `cirno backup` to create a backup of a workspace, and use `cirno restore` to restore to a backup workspace.

For example:

```sh
$ cirno backup A
> Successfully created a backup workspace B.
$ cirno backup A
> Successfully created a backup workspace C.
$ cirno backup A
> Successfully created a backup workspace D.
```

This will create four workspaces with the following relationship:

```
B -> C -> D -> A
```

Workspace A is called the *head* workspace, and workspaces B, C, D are called *base* workspaces. These two types of workspaces have different behaviors:

- You cannot `backup` a base workspace. For example, the following command will fail:

  ```sh
  $ cirno backup D
  > Cannot backup a base workspace.
  ```

  Despite this, you can still `clone` a base workspace.

- You cannot `restore` to a head workspace. For example, the following command will fail:

  ```sh
  $ cirno restore A
  > Cannot restore to a head workspace.
  ```

- When you `restore` to a base workspace, all the following workspaces will be removed. For example, if you `restore` to workspace C, the backup graph will be:

  ```
  B -> C
  ```

  In this case, C will become the head workspace on which you can make further backups.

- When you `remove` a base workspace, the next node in the backup graph will be linked to the previous node. For example, if you `remove` workspace C, the backup graph will be:

  ```
  B -> D -> A
  ```

- When you `remove` a head workspace, all base workspaces will be removed as well.


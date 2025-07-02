# Cirno

Zero-Install Package Manager for Node.js.

## Features

- Zero-Install
- Shared Cache with GC

## Usage

### `cirno init`

Initialize a new Cirno project.

### `cirno import <src>`

Import a workspace from a local path or URL.

### `cirno export <id>`

Export a workspace to a local path.

### `cirno clone <id>`

Clone a workspace.

### `cirno backup <id>`

Backup a workspace.

### `cirno restore <id>`

Restore to a backup workspace.

### `cirno remove <id>`

Remove a workspace.

### `cirno list`

List all workspaces in the project.

### `cirno show <id>`

Show the information of a workspace.

### `cirno yarn <id> -- [args]`

Execute `yarn` in a workspace.

### `cirno gc`

Remove unused packages from the cache.

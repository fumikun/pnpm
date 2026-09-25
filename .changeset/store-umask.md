---
"pacquet": minor
---

Added the `storeUmask` setting, which sets the permissions of files that pnpm writes to the store. With `storeUmask: 002`, new store files get mode `664` (or `775` for executables), whatever the umask of the process that runs the install. Files in `node_modules` are hardlinks to store files, so they get the same mode. Without the setting, the umask of the first install that adds a file decides its mode for every later project. Set it in the global `config.yaml` or with `PNPM_CONFIG_STORE_UMASK`. Files already in the store keep their mode [#3807](https://github.com/pnpm/pnpm/issues/3807).

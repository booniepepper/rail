## Takeaways from [So You Want to Write a Package Manager](https://medium.com/@sdboyer/so-you-want-to-write-a-package-manager-4ae9c17d9527)

1. Probably suggest using semver.
    - It has downsides (see: Rich Hickey's "Spec and Speculation") but it's better than no scheme
    - Major version change = Compatibility is broken (Hickey: Might as well change the project name)
    - Minor version change = Some feature was added (e.g. For rail or dt, a new command)
    - Patch version change = Internal changes (e.g. Some bug was fixed)
2. No version ranges(?)
3. Make a central repository
    - Namespaces (URLs?)
    - Websites
    - Accounts and MFA
    - Storage    
4. Make a project file format
    - TOML? (What about KDL? EDN is niche by nice)
    - Rail/Dt? We can interpret too!

CLI may need ideas of:
- init
- clean
- add <dependency>
- rm <dependency>
- install <application>

## Other ideas

Need a module system. Import and export commands
- Maybe a starting point: https://en.wikipedia.org/wiki/Modular_programming

## Other TODO

- Re-watch Rich Hickey's "Spec and Speculation" and take notes
- Read up on Nix philosophy and theory

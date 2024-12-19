## Rustimoji

An attempt to rewrite [rofimoji](https://github.com/fdw/rofimoji) in rust.
The main goal is to improve startup time.

#### Appraoch

- Load files from rofimoji to match available emoji
- Load files from rofimoji to build cache, which should speed up all but the first run

#### Dependencies

- Depends on rofi, and (currently) xclip

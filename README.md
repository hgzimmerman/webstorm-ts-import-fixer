# webstorm-ts-import-fixer
Brings ts files that have use idea's autoimport feature in line with the rest of our codebase.

It will recurse into the project's directory structure, altering any .ts file with "nonstandard" imports. Converting
* `import {Namespace} from 'file/path'` to `import { Namespace } from 'file/path'`.
* `import { Namespace } from "file/path"` to `import { Namespace } from 'file/path'`. 
And moving entries in or after the logging section into the normal import section.

# Install
* Install rust `curl https://sh.rustup.rs -sSf | sh`.
* Add rust's directory install path to your $PATH `echo "source $HOME/.cargo/env" >> ~/.bashrc`.
* Clone this project.
* Run `cargo install` from the project directory.
* You can now run `webstorm-ts-import-fixer` to fix the formatting of the imports that webstorm uses.
* Congradulations, you now just pulled in another depency!

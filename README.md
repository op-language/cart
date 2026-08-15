# cart

The Op build tool and package manager.

`cart` manages Op projects the same way `cargo` manages Rust projects. It
reads and writes the `Cart.toml` manifest, resolves dependencies from
`~/.carts/`, invokes `opc` to compile projects, and installs libs from a
git-based registry.

## Subcommands

| Command | Description |
|---------|-------------|
| `cart init` | Create a new Op project |
| `cart build` | Compile the project and write the ROM image |
| `cart run` | Build the project and launch the ROM in an emulator |
| `cart test` | Run the project test suite |
| `cart check` | Run the lexer and parser without code generation |
| `cart clean` | Remove the build output directory |
| `cart add` | Add a lib to Cart.toml dependencies |
| `cart doc` | Generate documentation from doc comments |
| `cart install` | Install a lib in ~/.carts/ |
| `cart update` | Update all dependencies to the latest version |

## License

Apache-2.0
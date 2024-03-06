# babelone

> pronounced /ˌbæbɪˈloʊn/

## Wait, babe-what?

It's a Python CLI, built in Rust, that converts between requirements.txt, setup.py, or pyproject.toml while leaving the original file intact.

babelone was built to help you sort out all your package build spec files (or at least for inital scaffolding), because—let's face it—it can sometimes be a pain in the you-know-what.

## Python CLI Built in Rust?

There's not a whole lot of Python, but it is the layer that allows you to `pip install` with ease and talk to a friendly CLI (thanks to the folks at [rich-click](https://github.com/ewels/rich-click)).

The rest is indeed in Rust; and if that's still making your head itch, I recommend taking a look at these revolutionary projects:
-  [PyO3](https://github.com/PyO3/pyo3)
-  [RustPython](https://github.com/RustPython/RustPython)

## Installation

```bash
pip install babelone
```

## Usage

```bash
babelone --help

babelone <path>/[requirements.txt|setup.py|pyproject.toml] <path>/[requirements.txt|setup.py|pyproject.toml]
```

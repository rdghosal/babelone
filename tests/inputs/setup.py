import setuptools

VERSION: str = "2.0"
REQUIRES = [
    "pydantic==2.6.2",
    "fastapi",
]


if __name__ == "__main__":
    setuptools.setup(
        name="babelone-test",
        version=VERSION,
        author="Rahul D. Ghosal",
        install_requires=REQUIRES
    )

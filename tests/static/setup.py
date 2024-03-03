import setuptools

VERSION = "2.0"


if __name__ == "__main__":
    setuptools.setup(
        name="babelone-test",
        version=VERSION,
        author="Rahul D. Ghosal",
        install_requires=[
            "pydantic==2.6.2",
            "fastapi",
        ],
    )

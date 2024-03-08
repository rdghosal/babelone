"""Installation configuration and package metadata for spam-eggs."""
from setuptools import setup


if __name__ == "__main__":
    setup(
        package_name="spam-eggs",
        version="2020.0.0",
        install_requires=["httpx", "gidgethub[httpx]>4.0.0", "django>2.1; os_name != 'nt'", "django>2.0; os_name == 'nt'"],
        setup_requires=["hatchling"],
        extra_requires={"cli": ["rich", "click"], "gui": ["PyQt5"]},
        entry_points={"console_scripts": ["spam-cli = spam:main_cli"], "gui_scripts": ["spam-gui = spam:main_gui"]},
    )
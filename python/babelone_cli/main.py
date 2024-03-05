from babelone_cli import _babelone_core
import rich_click as click


@click.command(name="babelone")
@click.rich_config(
    help_config=click.RichHelpConfiguration(
        use_markdown=True, width=60, show_arguments=True
    )
)
@click.argument("source", required=True)
@click.argument("destination", required=True)
def translate(source: str, destination: str):
    """Translates a Python package build specification file from one
    format to another.

    e.g., setup.py -> pyproject.toml.

    """
    _babelone_core.translate(source, destination)


translate()

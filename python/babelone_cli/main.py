from babelone_cli import _babelone_core
import rich_click as click


@click.group(context_settings=dict(help_option_names=["-h", "--help"]))
@click.rich_config(
    help_config=click.RichHelpConfiguration(
        use_markdown=True, width=60, show_arguments=True
    )
)
@click.version_option(package_name="babelone", prog_name="babelone")
def cli():
    """babelone /ˌbæbɪˈloʊn/

    Scaffold or translate between Python package build specification files,
    including requirements.txt, setup.py, and pyproject.toml.

    """


@cli.command()
@click.argument("output", nargs=1, type=click.Path(), required=True)
def create(output: str):
    """Scaffold a build spec file and save at the OUTPUT path."""
    _babelone_core.create(output)


@cli.command()
@click.argument("input", nargs=1, type=click.Path(exists=True), required=True)
@click.argument("output", nargs=1, type=click.Path(), required=True)
def translate(input: str, output: str):
    """Translate the file at the INPUT path to another format saved at
    the OUTPUT path.

    """
    _babelone_core.translate(input, output)


cli()

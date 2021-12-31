from rich.style import Style
from rich.table import Table
from rich.live import Live
from rich.spinner import Spinner
from rich.progress import (
	BarColumn,
	DownloadColumn,
	Progress,
	TextColumn,
	TransferSpeedColumn,
	ProgressColumn,
	Text,
	filesize,
	Optional,
	Column,
	Console
)
console = Console()
rich_live = Live
rich_grid = Table().grid
rich_spinner = Spinner
rich_table = Table

class TransferSpeedColumn(ProgressColumn):
	"""Renders human readable transfer speed."""

	def render(self, task) -> Text:
		"""Show data transfer speed."""
		speed = task.finished_speed or task.speed
		if speed is None:
			return Text("?", style="progress.data.speed")
		data_speed = filesize.decimal(int(speed))
		return Text(f"{data_speed}/s", style="bold blue")

class DownloadColumn(ProgressColumn):
	"""Renders file size downloaded and total, e.g. '0.5/2.3 GB'.

	Args:
		binary_units (bool, optional): Use binary units, KiB, MiB etc. Defaults to False.
	"""

	def __init__(
		self, binary_units: bool = False, table_column: Optional[Column] = None
	) -> None:
		self.binary_units = binary_units
		super().__init__(table_column=table_column)

	def render(self, task) -> Text:
		"""Calculate common unit for completed and total."""
		completed = int(task.completed)
		total = int(task.total)
		if self.binary_units:
			unit, suffix = filesize.pick_unit_and_suffix(
				total,
				["bytes", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"],
				1024,
			)
		else:
			unit, suffix = filesize.pick_unit_and_suffix(
				total, ["bytes", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"], 1000
			)
		completed_ratio = completed / unit
		total_ratio = total / unit
		precision = 0 if unit == 1 else 1
		completed_str = f"{completed_ratio:,.{precision}f}"
		total_str = f"{total_ratio:,.{precision}f}"
		download_status = f"{completed_str}/{total_str} {suffix}"
		return Text(download_status, style="bold green")

bar_back_style = Style(color='red')
bar_style = Style(color='cyan')
pkg_download_progress = Progress(
	TextColumn("[bold blue]Downloading ...", justify="right"),
	BarColumn(
		bar_width=None,
		# The background of our bar
		style=bar_back_style,
		# The color completed section
		complete_style=bar_style,
		# The color of completly finshed bar
		finished_style=bar_style
	),
	"[progress.percentage][bold blue]{task.percentage:>3.1f}%",
	"[bold]•",
	DownloadColumn(),
	"[bold]•",
	TransferSpeedColumn(),
	)

fetch_progress = Progress(
	#TextColumn("[bold blue]Downloading ...", justify="right"),
	BarColumn(
	bar_width=None,
	# The background of our bar
	style=bar_back_style,
	# The color completed section
	complete_style=bar_style,
	# The color of completly finshed bar
	finished_style=bar_style),
)

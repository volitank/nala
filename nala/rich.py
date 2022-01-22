#                 __
#    ____ _____  |  | _____
#   /    \\__  \ |  | \__  \
#  |   |  \/ __ \|  |__/ __ \_
#  |___|  (____  /____(____  /
#       \/     \/          \/
#
# Copyright (C) 2021, 2022 Blake Lee
#
# This file is part of nala
#
# nala is program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# nala is program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with nala.  If not, see <https://www.gnu.org/licenses/>.
"""Rich options for Nala output."""
from __future__ import annotations

from rich.console import Console
from rich.live import Live
from rich.progress import (BarColumn, DownloadColumn, Progress,
				SpinnerColumn, Task, TextColumn, TransferSpeedColumn, filesize)
from rich.spinner import Spinner
from rich.style import Style
from rich.table import Column, Table
from rich.text import Text

__all__ = ('Spinner', 'Table', 'Column', 'Live', 'Text')

class NalaTransferSpeed(TransferSpeedColumn): # type: ignore[misc]
	"""Subclass of TransferSpeedColumn."""

	def render(self, task: Task) -> Text:
		"""Show data transfer speed."""
		speed = task.finished_speed or task.speed
		if speed is None:
			return Text("?", style="progress.data.speed")
		data_speed = filesize.decimal(int(speed))
		return Text(f"{data_speed}/s", style="bold blue")

class NalaDownload(DownloadColumn): # type: ignore[misc]
	"""Subclass of DownloadColumn."""

	def render(self, task: Task) -> Text:
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
console = Console()

pkg_download_progress = Progress(
	TextColumn("[bold blue]Downloading ...", justify="right"),
	BarColumn(
		bar_width=None,
		# The background of our bar
		style=bar_back_style,
		# The color completed section
		complete_style=bar_style,
		# The color of completely finished bar
		finished_style=bar_style
	),
	"[progress.percentage][bold blue]{task.percentage:>3.1f}%",
	"[bold]•",
	NalaDownload(),
	"[bold]•",
	NalaTransferSpeed(),
	)

dpkg_progress = Progress(
	SpinnerColumn(style="bold white"),
	TextColumn("[bold blue]Running dpkg ...", justify="right"),
	BarColumn(
		bar_width=None,
		# The background of our bar
		style=bar_back_style,
		# The color completed section
		complete_style=bar_style,
		# The color of completely finished bar
		finished_style=bar_style
	),
	"[progress.percentage][bold blue]{task.percentage:>3.1f}%",
	"{task.completed}/{task.total}"
)

fetch_progress = Progress(
	#TextColumn("[bold blue]Downloading ...", justify="right"),
	BarColumn(
	bar_width=None,
	# The background of our bar
	style=bar_back_style,
	# The color completed section
	complete_style=bar_style,
	# The color of completely finished bar
	finished_style=bar_style),
)

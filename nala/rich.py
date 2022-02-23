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

import sys
from datetime import timedelta

from rich.columns import Columns

try:
	from rich.console import Group  # type: ignore[attr-defined]
# Rich 11.0.0 changed RenderGroup to Group
# python3-rich in debian bullseye is 9.11.0
except ImportError:
	from rich.console import RenderGroup as Group  # type: ignore[attr-defined, no-redef]

from rich.ansi import AnsiDecoder
from rich.console import Console
from rich.live import Live
from rich.markup import escape
from rich.progress import (BarColumn, DownloadColumn, Progress, SpinnerColumn,
				Task, TextColumn, TimeRemainingColumn, TransferSpeedColumn, filesize)
from rich.spinner import Spinner
from rich.style import Style
from rich.table import Column, Table
from rich.text import Text
from rich.tree import Tree

__all__ = (
	'Spinner', 'Table',
	'Column', 'Columns',
	'Console', 'Tree',
	'Live', 'Text',
	'escape', 'Group'
)

# pylint: disable=too-few-public-methods
class NalaTransferSpeed(TransferSpeedColumn): # type: ignore[misc]
	"""Subclass of TransferSpeedColumn."""

	def render(self, task: Task) -> Text:
		"""Show data transfer speed."""
		speed = task.speed
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

class TimeRemaining(TimeRemainingColumn): # type: ignore[misc]
	"""Renders estimated time remaining."""

	def render(self, task: Task) -> Text:
		"""Show time remaining."""
		remaining = task.time_remaining
		if remaining is None:
			return Text("-:--:--", style="bold white")
		remaining_delta = timedelta(seconds=int(remaining))
		return Text(str(remaining_delta), style="white")

bar_back_style = Style(color='red')
bar_style = Style(color='cyan')
# Perform checks to see if we need to fall back to ascii.
is_utf8 = sys.stdout.encoding == 'utf-8'
SEPARATOR = "[bold]•" if is_utf8 else "[bold]+"
SPIN_TYPE = 'dots' if is_utf8 else 'simpleDots'
FINISHED_TEXT = "[bold green]:heavy_check_mark:" if is_utf8 else " "
PROGRESS_PERCENT = "[bold blue]{task.percentage:>3.1f}%"
COMPLETED_TOTAL = "{task.completed}/{task.total}"

def from_ansi(msg: str) -> Text:
	"""Convert ansi coded text into Rich Text."""
	return Text().join(AnsiDecoder().decode(msg))

def ascii_replace(string: str) -> str:
	"""If terminal is in ascii mode replace unicode characters."""
	return string if is_utf8 else string.encode('ascii', 'replace').decode('ascii')

spinner = Spinner(
	SPIN_TYPE,
	text='Initializing',
	style="bold blue"
)

pkg_download_progress = Progress(
	TextColumn("[bold green]Time Remaining:"),
	TimeRemaining(),
	BarColumn(
		bar_width=None,
		# The background of our bar
		style=bar_back_style,
		# The color completed section
		complete_style=bar_style,
		# The color of completely finished bar
		finished_style=bar_style
	),
	PROGRESS_PERCENT,
	SEPARATOR,
	NalaDownload(),
	SEPARATOR,
	NalaTransferSpeed(),
	)

dpkg_progress = Progress(
	SpinnerColumn(SPIN_TYPE, style="bold white", finished_text=FINISHED_TEXT),
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
	PROGRESS_PERCENT,
	SEPARATOR,
	TimeRemaining(),
	SEPARATOR,
	COMPLETED_TOTAL
)

search_progress = Progress(
	SpinnerColumn(SPIN_TYPE, style="bold blue"),
	TextColumn("[bold white]Searching ...", justify="right"),
	BarColumn(
		#bar_width=None,
		# The background of our bar
		style=bar_back_style,
		# The color completed section
		complete_style=bar_style,
		# The color of completely finished bar
		finished_style=bar_style
	),
	PROGRESS_PERCENT,
	SEPARATOR,
	TimeRemaining(),
	transient=True
)

fetch_progress = Progress(
	SpinnerColumn(SPIN_TYPE, style="bold blue"),
	TextColumn("[bold white]Testing Mirrors ...", justify="right"),
	BarColumn(
		#bar_width=None,
		# The background of our bar
		style=bar_back_style,
		# The color completed section
		complete_style=bar_style,
		# The color of completely finished bar
		finished_style=bar_style
	),
	PROGRESS_PERCENT,
	SEPARATOR,
	COMPLETED_TOTAL,
	transient=True
)

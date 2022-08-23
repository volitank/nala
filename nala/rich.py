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
# nala is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# nala is distributed in the hope that it will be useful,
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
from typing import Literal, cast

from rich import filesize
from rich.ansi import AnsiDecoder
from rich.box import Box
from rich.columns import Columns
from rich.console import Console, Group, RenderableType
from rich.live import Live, _RefreshThread
from rich.markup import escape
from rich.panel import Panel
from rich.pretty import Pretty
from rich.progress import (
	BarColumn,
	DownloadColumn,
	Progress,
	SpinnerColumn,
	Task,
	TaskID,
	TextColumn,
	TimeRemainingColumn,
	TransferSpeedColumn,
)
from rich.spinner import Spinner
from rich.style import Style
from rich.table import Column, Table
from rich.text import Text
from rich.tree import Tree

from nala import _, console
from nala.options import arguments

__all__ = (
	"Spinner",
	"Table",
	"Column",
	"Columns",
	"Console",
	"Tree",
	"Live",
	"Text",
	"escape",
	"Group",
	"TaskID",
	"Panel",
	"Pretty",
	"Progress",
	"RenderableType",
)


class Thread(_RefreshThread):
	"""A thread that calls refresh() at regular intervals.

	Subclass to change live.refresh with live.update.
	"""

	def run(self) -> None:
		while not self.done.wait(1 / self.refresh_per_second):
			with self.live._lock:
				if not self.done.is_set():
					self.live.scroll_bar(rerender=True)  # type: ignore[attr-defined]


def to_str(
	size: int,
	base: int,
) -> str:
	"""Format transfer speed to a string."""
	if arguments.config.get_bool("transfer_speed_bits", False):
		single = "bits"
		suffixes = ("Kbit", "Mbit", "Gbit", "Tbit", "Pbit", "Ebit", "Zbit", "Ybit")
		multiplier = 8
	else:
		single = "bytes"
		suffixes = ("KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB")
		multiplier = 1

	if size == 1:
		return f"1 {single}"
	if size < base:
		return f"{size:,} {single}"

	for i, suffix in enumerate(suffixes, 2):
		unit = base**i
		if size < unit:
			break

	display = (base * size / unit) * multiplier
	return f"{display:,.1f} {suffix}"


# pylint: disable=too-few-public-methods
class NalaTransferSpeed(TransferSpeedColumn):  # type: ignore[misc]
	"""Subclass of TransferSpeedColumn."""

	def render(self, task: Task) -> Text:
		"""Show data transfer speed."""
		if (speed := task.speed) is None:
			return Text("?", style="progress.data.speed")
		return Text(f"{to_str(int(speed), 1000)}/s", style="bold blue")


class NalaDownload(DownloadColumn):  # type: ignore[misc]
	"""Subclass of DownloadColumn."""

	def render(self, task: Task) -> Text:
		"""Calculate common unit for completed and total."""
		completed = int(task.completed)
		total = int(cast(float, task.total))  # type: ignore[redundant-cast]

		if arguments.config.get_bool("filesize_binary", False):
			unit, suffix = filesize.pick_unit_and_suffix(
				total,
				["bytes", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"],
				1024,
			)
		else:
			unit, suffix = filesize.pick_unit_and_suffix(
				total,
				["bytes", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"],
				1000,
			)

		completed_ratio = completed / unit
		total_ratio = total / unit
		precision = 0 if unit == 1 else 1
		completed_str = f"{completed_ratio:,.{precision}f}"
		total_str = f"{total_ratio:,.{precision}f}"
		download_status = f"{completed_str}/{total_str} {suffix}"
		return Text(download_status, style="bold green")


class TimeRemaining(TimeRemainingColumn):  # type: ignore[misc]
	"""Renders estimated time remaining."""

	def render(self, task: Task) -> Text:
		"""Show time remaining."""
		remaining = task.time_remaining
		if remaining is None:
			return Text("-:--:--", style="bold default")
		remaining_delta = timedelta(seconds=int(remaining))
		return Text(f"{remaining_delta}", style="")


bar_back_style = Style(color="red")
bar_style = Style(color="cyan")
# Perform checks to see if we need to fall back to ascii.
is_utf8 = sys.stdout.encoding == "utf-8"
SEPARATOR = "[bold]•" if is_utf8 else "[bold]+"
SPIN_TYPE = "dots" if is_utf8 else "simpleDots"
FINISHED_TEXT = "[bold green]:heavy_check_mark:" if is_utf8 else " "
PROGRESS_PERCENT = "[bold blue]{task.percentage:>3.1f}%"
COMPLETED_TOTAL = "{task.completed}/{task.total}"
ELLIPSIS = "…" if is_utf8 else "..."
OVERFLOW = cast(Literal["crop"], "crop" if console.options.ascii_only else "ellipsis")

HORIZONTALS = Box(
	"\n".join(
		(
			"====",
			"    ",
			"====",
			"    ",
			"====",
			"====",
			"    ",
			"    ",
		)
	),
	ascii=True,
)

BAR_MAX = BarColumn(
	bar_width=None,
	# The background of our bar
	style=bar_back_style,
	# The color completed section
	complete_style=bar_style,
	# The color of completely finished bar
	finished_style=bar_style,
)
BAR_MIN = BarColumn(
	# The background of our bar
	style=bar_back_style,
	# The color completed section
	complete_style=bar_style,
	# The color of completely finished bar
	finished_style=bar_style,
)


def from_ansi(msg: str) -> Text:
	"""Convert ansi coded text into Rich Text."""
	return Text().join(AnsiDecoder().decode(msg))


def ascii_replace(string: str) -> str:
	"""If terminal is in ascii mode replace unicode characters."""
	return string if is_utf8 else string.encode("ascii", "replace").decode("ascii")


spinner = Spinner(SPIN_TYPE, style="bold blue")
time_remain = _("Time Remaining:")
pkg_download_progress = Progress(
	TextColumn(f"[bold green]{time_remain}"),
	TimeRemaining(),
	BAR_MAX,
	PROGRESS_PERCENT,
	SEPARATOR,
	NalaDownload(),
	SEPARATOR,
	NalaTransferSpeed(),
)
running_dpkg = _("Running dpkg")
dpkg_progress = Progress(
	SpinnerColumn(SPIN_TYPE, style="bold default", finished_text=FINISHED_TEXT),
	TextColumn(f"[bold blue]{running_dpkg} {ELLIPSIS}", justify="right"),
	BAR_MAX,
	PROGRESS_PERCENT,
	SEPARATOR,
	TimeRemaining(),
	SEPARATOR,
	COMPLETED_TOTAL,
)
testing = _("Testing Mirrors")
fetch_progress = Progress(
	# 	SpinnerColumn(SPIN_TYPE, style="bold blue"),
	TextColumn(f"[bold blue]{testing} {ELLIPSIS}", justify="right"),
	BAR_MIN,
	PROGRESS_PERCENT,
	SEPARATOR,
	COMPLETED_TOTAL,
	transient=True,
)

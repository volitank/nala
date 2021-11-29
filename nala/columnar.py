import shutil
import re
import io
import operator
from functools import reduce
from itertools import zip_longest
from typing import (
    Union,
    Tuple,
    Sequence,
    List,
    Any,
)
import collections
from wcwidth import wcswidth

# Defining this from the toolz package to lower dependencies during install
def frequencies(seq):
    """ Find number of occurrences of each value in seq

    >>> frequencies(['cat', 'cat', 'ox', 'pig', 'pig', 'cat'])  #doctest: +SKIP
    {'cat': 3, 'ox': 1, 'pig': 2}

    See Also:
        countby
        groupby
    """
    d = collections.defaultdict(int)
    for item in seq:
        d[item] += 1
    return dict(d)

class TableOverflowError(Exception):
    pass

# Types
NonWrappedCell = str
WrappedCellLine = str
Data = List[List[NonWrappedCell]]
Headers = List[str]
LogicalRow = List[List[WrappedCellLine]]

class Columnar:
    def __call__(
        self,
        data: Sequence[Sequence[Any]],
        headers: Union[None, Sequence[Any]] = None,
        head: int = 0,
        justify: str = "l",
        wrap_max: int = 5,
        unjust: bool = False,
        max_column_width: Union[None, int] = None,
        min_column_width: int = 5,
        row_sep: str = "-",
        column_sep: str = "|",
        patterns: Sequence[str] = [],
        drop: Sequence[str] = [],
        select: Sequence[str] = [],
        no_borders: bool = False,
        terminal_width: Union[None, int] = None,
    ) -> str:
        self.wrap_max = wrap_max
        self.max_column_width = max_column_width
        self.min_column_width = min_column_width
        self.justify = justify
        self.head = head
        self.unjust = unjust
        self.terminal_width = (
            terminal_width
            if terminal_width is not None
            else shutil.get_terminal_size().columns
        )
        self.row_sep = row_sep
        self.column_sep = column_sep
        self.header_sep = "="
        self.patterns = self.compile_patterns(patterns)
        self.ansi_color_pattern = re.compile(r"\x1b\[.+?m")
        self.color_reset = "\x1b[0m"
        self.color_grid = None
        self.drop = drop
        self.select = select
        self.no_borders = no_borders
        self.no_headers = headers is None

        if self.no_headers:
            headers = [""] * len(data[0])

        if self.no_borders:
            self.column_sep = " " * 2
            self.row_sep = ""
            self.header_sep = ""

        data = self.clean_data(data)
        data, headers = self.filter_columns(data, headers)

        if self.no_headers:
            logical_rows = self.convert_data_to_logical_rows(data)
        else:
            logical_rows = self.convert_data_to_logical_rows([headers] + data)

        column_widths = self.get_column_widths(logical_rows)
        truncated_rows = self.wrap_and_truncate_logical_cells(
            logical_rows, column_widths
        )
        justification_map = {
            "l": lambda text, width: self.visual_justify(text, width, 'l'),
            "c": lambda text, width: self.visual_justify(text, width, 'c'),
            "r": lambda text, width: self.visual_justify(text, width, 'r'),
        }
        justifications = []
        if type(justify) is str:
            justifications = [justification_map[justify]] * len(column_widths)
        else:
            justifications = [justification_map[spec] for spec in justify]

        table_width = sum(column_widths) + ((len(column_widths) + 1) * len(row_sep))
        out = io.StringIO()
        write_header = True if not self.no_headers else False
        self.write_row_separators(out, column_widths)
        for lrow, color_row in zip(truncated_rows, self.color_grid):
            for row in lrow:
                justified_row_parts = [
                    justifier(text, width)
                    for text, justifier, width in zip(
                        row, justifications, column_widths
                    )
                ]
                colorized_row_parts = [
                    self.colorize(text, code)
                    for text, code in zip(justified_row_parts, color_row)
                ]
                if self.unjust:
                    out.write(
                        self.column_sep.join(colorized_row_parts)
                        + self.column_sep * 2
                        + "\n"
                    )
                    self.unjust = False
                else:
                    out.write(
                        self.column_sep +
                        self.column_sep.join(colorized_row_parts)
                        + self.column_sep
                        + "\n"
                    )
            if write_header:
                out.write(
                    (self.header_sep * (table_width - (len(self.column_sep * 2))))
                )
                write_header = False
            else:
                if not self.no_borders:
                    self.write_row_separators(out, column_widths)

        return out.getvalue()

    def write_row_separators(
        self, out_stream: io.StringIO, column_widths: Sequence[int]
    ) -> None:
        cells = [self.row_sep * width for width in column_widths]
        out_stream.write(
            self.column_sep + self.column_sep.join(cells) + self.column_sep + "\n"
        )

    def compile_patterns(self, patterns):
        out = []
        for regex, func in patterns:
            if regex is not re.Pattern:
                regex = re.compile(regex)
            out.append((regex, func))
        return out

    def colorize(self, text, code):
        if code == None:
            return text
        return "".join([code, text, self.color_reset])

    def clean_data(self, data: Sequence[Sequence[Any]]) -> Data:
        # First make sure data is a list of lists
        if type(data) is not list:
            raise TypeError(f"'data' must be a list of lists. Got a {type(data)}")
        if type(data[0]) is not list:
            raise TypeError(f"'data' must be a list of lists. Got a list of {type(data[0])}")
        # Make sure all the lists are the same length
        num_columns = len(data[0])
        for row_num, row in enumerate(data):
            if len(row) != num_columns:
                raise ValueError(
                    f"All the rows in 'data' must have the same number of columns, however the first row had {num_columns} columns and row number {row_num + 1} had {len(row)} column(s)."
                )
        carriage_return = re.compile("\r")
        tab = re.compile("\t")
        out = []
        for row in data:
            cleaned = []
            for cell in row:
                cell = str(cell)
                cell = carriage_return.sub("", cell)
                cell = tab.sub(" " * 4, cell)
                cleaned.append(cell)
            out.append(cleaned)
        return out

    def filter_columns(self, data: Data, headers: Headers) -> Tuple[Data, Headers]:
        """
        Drop columns that meet drop criteria, unless they have been
        explicitly selected.
        """
        drop = set(self.drop)
        select_patterns = [re.compile(pattern, re.I) for pattern in self.select]
        select = len(select_patterns) > 0
        headers_out = []
        columns_out = []
        for header, column in zip(headers, zip(*data)):
            if select:
                for pattern in select_patterns:
                    if pattern.search(header):
                        headers_out.append(header)
                        columns_out.append(column)
            else:
                freqs = frequencies(column)
                if not set(freqs.keys()).issubset(drop):
                    headers_out.append(header)
                    columns_out.append(column)
        rows_out = list(zip(*columns_out))
        return rows_out, headers_out

    def convert_data_to_logical_rows(self, full_data: Data) -> List[LogicalRow]:
        """
        Takes a list of lists of items. Returns a list of logical rows, where each logical
        row is a list of lists, where each sub-list in a logical row is a physical row to be
        printed to the screen. There will only be more than one phyical row in a logical
        row if one of the columns wraps past one line. However, wrapping will be performed
        in a later step, so this function always returns logical rows that only contain
        one physical row which will be wrapped onto multiple physical rows later.
        """
        logical_rows = []
        color_grid = []

        for row in full_data:
            cells_varying_lengths = []
            color_row = []
            for cell in row:
                cell = self.apply_patterns(cell)
                cell, color = self.strip_color(cell)
                color_row.append(color)
                lines = cell.split("\n")
                cells_varying_lengths.append(lines)
            cells = [
                [cell_text or "" for cell_text in physical_row]
                for physical_row in zip_longest(*cells_varying_lengths)
            ]
            logical_rows.append(cells)
            color_grid.append(color_row)
        self.color_grid = color_grid
        return logical_rows

    def apply_patterns(self, cell_text):
        out_text = cell_text
        for pattern, func in self.patterns:
            if pattern.match(cell_text):
                out_text = func(cell_text)
                break
        return out_text

    def strip_color(self, cell_text):
        matches = [match for match in self.ansi_color_pattern.finditer(cell_text)]
        color_codes = None
        clean_text = cell_text
        if matches:
            clean_text = self.ansi_color_pattern.sub("", cell_text)
            color_codes = "".join([match.group(0) for match in matches[:-1]])
        return clean_text, color_codes

    def distribute_between(self, diff: int, columns: List[dict], n: int) -> List[dict]:
        """
        Reduces the total width of the n widest columns by 'diff', returning
        the list of columns such that the first n columns are now all the 
        same width. This function will continue to be called as long as the nth 
        column is narrower than the n+1 th column, meaning that we could still
        distribute our 'diff' more equally among the widest columns.
        """
        subset = columns[:n]
        width = sum([column["width"] for column in subset])
        remainder = width - diff
        new_width = remainder // n
        for i in range(n):
            columns[i]["width"] = new_width
        return columns

    def widths_sorted_by(self, columns: List[dict], key: str) -> List[int]:
        return [column["width"] for column in sorted(columns, key=lambda x: x[key])]

    def current_table_width(self, columns: List[dict]) -> int:
        return sum(
            [len(self.column_sep) + column["width"] for column in columns]
        ) + len(self.column_sep)

    def get_column_widths(self, logical_rows: List[LogicalRow]) -> List[int]:
        """
        Calculated column widths, taking into account the terminal width,
        the number of columns, and the column seperators that will be used
        to delimit columns.

        Our table-sizing heuristic says that we should keep wide
        columns as wide as possible and only touch narrow columns if we have shrunken
        the wide columns down to the width of the narrow columns and the table is still
        too wide to fit in the display.

        The function we will utilize to determine our column widths is
        'self.distribute_between'. It has three arguments:
        1. 'diff' is the size by which we need to shrink the table to get it 
        to fit in the terminal. 
        2. 'columns' is a list of dictionaries that
        represent the columns in the table, sorted from widest to narrowest.
        3. 'n' is the number of columns whose size will be reduced to reduce the table
        size by a total of 'diff'.
        The first time distribute_between is called n will be 1 and
        'diff' will be a positive value and the first/largest column's width will be 
        reduced by 'diff'. 

        Often the state of our table after the first call to 
        distribute_between does not follow our heuristic since the widest column is now 
        narrower, potentially much narrower, than the next widest column. (More formally speaking,
        the nth column is now narrower than the n+1 th column keeping in mind that the columns
        are sorted from widest to narrowest). It would be more
        desirable to shrink several wide columns a little bit than to shrink one column a lot.
        So, to "shrink several wide columns a little bit" we will
        redistribute the original "diff" amount between the widest columns. We will determine
        the number of columns to split the "diff" between by calling distribute_between 
        multiple times and adding the next-largest column into the group that shares the "diff". 
        After each call we will check if column n+1 is wider than the first n
        columns (which will all be the same width), and if so we will call distribute_between 
        again to ensure that we are shrinking columns equitably. Once column number n+1 is narrower 
        than the first n columns we are done.

        So starting with the second call to distribute_between 'diff' will be 0, but n will increase
        by one each call, meaning that the origial 'diff' amount will get distributed between a 
        larger number of columns each round until we either manage to get a table that fits and 
        preserves the order of column sizes, or we have exhausted our columns as we throw a 
        TableOverflowError.
        """

        max_widths = []
        for column in zip(*reduce(operator.add, logical_rows)):
            lengths = [len(cell) for cell in column]
            max_natural = max(lengths)
            max_width = (
                max_natural
                if self.max_column_width == None
                else min(max_natural, self.max_column_width)
            )
            max_widths.append(max_width)

        columns = sorted(
            [{"column_no": no, "width": width} for no, width in enumerate(max_widths)],
            key=lambda x: x["width"],
            reverse=True,
        )
        # apply min and max widths
        for column in columns:
            if column["width"] < self.min_column_width:
                column["width"] = self.min_column_width
            if self.max_column_width:
                if column["width"] > self.max_column_width:
                    column["width"] = self.max_column_width

        if self.current_table_width(columns) <= self.terminal_width:
            return self.widths_sorted_by(columns, "column_no")

        # the table needs to be narrowed
        for i in range(len(columns)):
            # include the next largest column in the size reduction
            diff = self.current_table_width(columns) - self.terminal_width
            columns = self.distribute_between(diff, columns, i + 1)
            if i < len(columns) - 1 and columns[0]["width"] < columns[i + 1]["width"]:
                # if the columns that were just shrunk are smaller than the next largest column,
                # keep distributing the size so we have evenly-shrunken columns
                continue
            elif (
                columns[0]["width"] >= self.min_column_width
                and self.current_table_width(columns) <= self.terminal_width
            ):
                return self.widths_sorted_by(columns, "column_no")

        raise TableOverflowError(
            "Could not fit table in current terminal, try reducing the number of columns."
        )

    def wrap_and_truncate_logical_cells(
        self, logical_rows: List[LogicalRow], column_widths: List[int]
    ) -> List[LogicalRow]:
        lrows_out = []
        for lrow in logical_rows:
            cells_out = []
            for cell, width in zip(map(list, zip(*lrow)), column_widths):
                # at this point `cell` is a list of strings, representing each line of the cell's contents
                cell_out = []
                for line in cell:
                    # Get the line width accounting for characters that occupy two terminal columns
                    # e.g. Unicode code point U+1F32D has a visual width of 2
                    while wcswidth(line) > width:
                        wrap_index = width
                        while wcswidth(line[:wrap_index]) > width:
                            # decrease the number of characters on the line until the 
                            # visual width is <= width.
                            wrap_index -= 1
                        cell_out.append(line[:wrap_index])
                        line = line[wrap_index:]
                    cell_out.append(line)
                cells_out.append(cell_out[: self.wrap_max + 1])
            cells_out_padded = [
                [text or "" for text in line] for line in zip_longest(*cells_out)
            ]
            lrows_out.append(cells_out_padded)
        return lrows_out

    def visual_justify(self, text: str, width: int, alignment: str) -> str:
        """
        The default python string methods, ljust, center, and rjust check
        the string length using len(), which adds too many spaces when the 
        string includes characters with a visual length of 2. We need to
        implement our own justification methods to handle this.
        """
        text_width = wcswidth(text)
        diff = width - text_width
        if alignment == 'l':
            right_padding = " " * diff
            return text + right_padding
        elif alignment == 'c':
            left_length = (diff // 2)
            left_padding = " " * left_length
            right_padding = " " * (diff - left_length)
            return ''.join([left_padding, text, right_padding])
        elif alignment == 'r':
            left_padding = " " * diff
            return left_padding + text
        else:
            raise ValueError(f"Got invalid justification value: {alignment}")
import argparse
from sys import argv
from pydoc import pager
from pathlib import Path

from nala.utils import LICENSE
from nala import __version__

# Custom Parser for printing help on error.
class nalaParser(argparse.ArgumentParser):
	def error(self, message):
		#stderr.write('error: %s\n' % message)
		self.print_help()
		exit(1)

# Subclassing the HelpFormatter just to fix an empty line from setting metavar=''
class nalaFormatter(argparse.RawDescriptionHelpFormatter):
#class nalaFormatter(argparse.HelpFormatter):
	def format_help(self):
		help = self._root_section.format_help()
		if help:
			help = self._long_break_matcher.sub('\n\n', help)
			help = help.strip('\n') + '\n'
		if 'command:' in help:
			help = help.replace('\n\ncommand:\n','\n\ncommand:')
		return help

	def _split_lines(self, text, width):
		if text.startswith('R|'):
			return text[2:].splitlines()  
		# this is the RawTextHelpFormatter._split_lines
		return argparse.RawTextHelpFormatter._split_lines(self, text, width)

# Custom Action for --license switch
class GPLv3(argparse.Action):
	def __init__(self,
			option_strings,
			dest=argparse.SUPPRESS,
			default=argparse.SUPPRESS,
			help='reads the GPLv3'):
		super(GPLv3, self).__init__(
			option_strings=option_strings,
			dest=dest,
			default=default,
			nargs=0,
			help=help)

	def __call__(self, parser, args, values, option_string=None):
		if LICENSE.exists():
			with open(LICENSE, 'r') as file:
				pager(file.read())
		else:
			print('It seems the system has no license file')
			print('Nala is licensed under the GPLv3')
			print('https://www.gnu.org/licenses/gpl-3.0.txt')
		parser.exit()

def remove_options(parser, yes=True, download=True, update=True):
	for item in parser._optionals._group_actions[:]:
		if yes:
			if '--assume-yes' in item.option_strings:
				parser._optionals._group_actions.remove(item)
		if download:			
			if '--download-only' in item.option_strings:
				parser._optionals._group_actions.remove(item)
		if update:		
			if '--no-update' in item.option_strings:
				parser._optionals._group_actions.remove(item)


# Main Parser
def arg_parse():

	formatter = lambda prog: nalaFormatter(prog, max_help_position=64)

	bin_name = Path(argv[0]).name

	version = __version__

	# Define global options to be given to subparsers
	global_options = nalaParser(add_help=False)
	global_options.add_argument('-y', '--assume-yes', action='store_true', help="assume 'yes' to all prompts and run non-interactively.")
	global_options.add_argument('-d', '--download-only', action='store_true', help="package files are only retrieved, not unpacked or installed")
	global_options.add_argument('-v', '--verbose', action='store_true', help='Logs extra information for debugging')
	global_options.add_argument('--no-update', action='store_true', help="skips updating the package list")
	global_options.add_argument('--debug', action='store_true', help='Logs extra information for debugging')
	global_options.add_argument('--version', action='version', version=f'{bin_name} {version}')
	global_options.add_argument('--license', action=GPLv3)

	parser = nalaParser(	formatter_class=formatter,
							usage=f'{bin_name} [--options] <command>',
							parents=[global_options]
							)

	# Define our subparser
	subparsers = parser.add_subparsers(metavar='', dest='command')
	parser._subparsers.title = "command"
	parser._optionals.title = "options"

	# Parser for the install command 
	install_parser = subparsers.add_parser('install', 
		help='install packages',
		parents=[global_options],
		usage=f'{bin_name} install [--options] pkg1 [pkg2 ...]'
		)
		
	install_parser.add_argument('args', metavar='pkg(s)', nargs='*', help='package(s) to install')
	install_parser._positionals.title = "arguments"
	install_parser._optionals.title = "options"

	# Parser for the remove command
	remove_parser = subparsers.add_parser('remove',
		help='remove packages', parents=[global_options],
		usage=f'{bin_name} remove [--options] pkg1 [pkg2 ...]'
	)

	# Remove Global options that I don't want to see in remove --help
	remove_options(remove_parser, yes=False)

	remove_parser.add_argument('args',
		metavar='pkg(s)',
		nargs='*',
		help='package(s) to remove')

	remove_parser._positionals.title = "arguments"
	remove_parser._optionals.title = "options"

	# We specify the options as a parent parser first just so we can easily move them above the global options inside the subparser help.
	# If there is a better way of doing this please let me know
	update_options = nalaParser(add_help=False)
	update_options.add_argument('--no-full', action='store_false', help="R|runs a normal upgrade instead of full-upgrade\n\n")

	# Parser for the update/upgrade command
	update_parser = subparsers.add_parser('update',
		formatter_class=formatter,
		help='update package list and upgrade the system',
		parents=[update_options, global_options],
		usage=f'{bin_name} update [--options]'
	)
	update_parser._optionals.title = "options"

	upgrade_parser = subparsers.add_parser('upgrade',
		formatter_class=formatter,
		help='alias for update',
		parents=[update_options, global_options],
		usage=f'{bin_name} upgrade [--options]'
	)
	upgrade_parser._optionals.title = "options"

	# We do the same thing that we did with update options
	fetch_options = nalaParser(add_help=False)
	fetch_options.add_argument('--fetches', metavar='number', type=int, default=3, help="number of mirrors to fetch")
	fetch_options.add_argument('--debian', metavar='sid', help="choose the Debian release")
	fetch_options.add_argument('--ubuntu', metavar='jammy', help="choose an Ubuntu release")
	fetch_options.add_argument('--country', metavar='"United States"', help="country must be as it appears on the mirror site. use quotes for spaces")
	fetch_options.add_argument('--foss', action='store_true', help="ommits contrib and non-free repos\n\n")

	# Parser for the fetch command
	fetch_parser = subparsers.add_parser('fetch',
		formatter_class=formatter,
		description=(
		'nala will fetch mirrors with the lowest latency.\n'
		'for Debian https://www.debian.org/mirror/list-full\n'
		'for Ubuntu https://launchpad.net/ubuntu/+archivemirrors'
		),
		help='fetches fast mirrors to speed up downloads',
		parents=[fetch_options, global_options],
		usage=f'{bin_name} fetch [--options]'
	)
	# Remove Global options that I don't want to see in fetch --help
	remove_options(fetch_parser)

	#fetch_parser.add_argument('fetches', nargs='?', default='3', type=int, help="1-10 of how many mirrors to fetch. default is 3")
	fetch_parser._positionals.title = "arguments"
	fetch_parser._optionals.title = "options"

	# Parser for the show command
	show_parser = subparsers.add_parser('show',
		help='show package details',
		parents=[global_options],
		usage=f'{bin_name} show [--options] pkg1 [pkg2 ...]'
	)
	# Remove Global options that I don't want to see in show --help
	remove_options(show_parser)

	show_parser._positionals.title = "arguments"
	show_parser._optionals.title = "options"
	show_parser.add_argument('args', metavar='pkg(s)', nargs='*', help='package(s) to show')

	# Parser for the History command
	history_parser = subparsers.add_parser('history', help='show transaction history',
		description='history without additional commands lists a history summary',
		parents=[global_options],
		usage=f'{bin_name} history [--options] <command> <id|all>'
	)

	# Remove Global options that I don't want to see in history --help
	remove_options(history_parser)

	history_parser._positionals.title = "commands"
	history_parser._optionals.title = "options"
	history_parser.add_argument('mode', metavar='info <id>', nargs='?', help='action you would like to do on the history')
	history_parser.add_argument('id', metavar='undo <id>', nargs='?', help='undo a transaction')
	history_parser.add_argument('placeholder', metavar='redo <id>', nargs='?', help='redo a transaction')
	history_parser.add_argument('placeholder2', metavar='clear <id>|all', nargs='?', help='clear a transaction or the entire history')

	# Parser for the show command
	clean_parser = subparsers.add_parser('clean',
		help='clears out the local repository of retrieved package files.',
		parents=[global_options],
		usage=f'{bin_name} show [--options]'
	)

	# Remove Global options that I don't want to see in clean --help
	remove_options(clean_parser)

	# This is just moo, but we can't cause are cat
	moo_parser = subparsers.add_parser('moo',
		description='nala is unfortunately unable to moo',
		parents=[global_options],
		usage=f'{bin_name} moo [--options]'
	)
	moo_parser.add_argument('moo', nargs='*', help=argparse.SUPPRESS)
	moo_parser._optionals.title = "options"

	return parser

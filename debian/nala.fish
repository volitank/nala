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
# completion for nala

function __fish_apt_no_subcommand -d 'Test if nala has yet to be given the subcommand'
    for i in (commandline -opc)
        if contains -- $i show search install remove purge update upgrade fetch history clean moo
            return 1
        end
    end
    return 0
end

function __fish_apt_use_package -d 'Test if nala command should have packages as potential completion'
    for i in (commandline -opc)
        if contains -- $i install purge remove show
            return 0
        end
    end
    return 1
end

complete -c nala -n __fish_apt_use_package -a '(__fish_print_apt_packages)' -d Package

complete -c nala -s h -l help -d 'Display a brief help message. Identical to the help action'
complete -f -n __fish_apt_no_subcommand -c nala -a install -d 'Install packages'
complete -f -n __fish_apt_no_subcommand -c nala -a remove -d 'Remove packages'
complete -f -n __fish_apt_no_subcommand -c nala -a update -d 'Update package list and upgrade the system'
complete -f -n __fish_apt_no_subcommand -c nala -a upgrade -d 'Alias for update'
complete -f -n __fish_apt_no_subcommand -c nala -a fetch -d 'Fetches fast mirrors to speed up downloads'
complete -f -n __fish_apt_no_subcommand -c nala -a history -d 'Show transaction history'
complete -f -n __fish_apt_no_subcommand -c nala -a purge -d 'Remove and delete all associated configuration and data files'
complete -f -n __fish_apt_no_subcommand -c nala -a clean -d 'Clears out the local repository of retrieved package files'
complete -f -n __fish_apt_no_subcommand -c nala -a show -d 'Display detailed information about the packages'
complete -f -n __fish_apt_no_subcommand -c nala -a search -d 'Search package names and descriptions'

complete -c nala -s h -l help -d 'Show help message'
complete -c nala -s y -l assume-yes -d 'Assume `yes` to all prompts and run non-interactively'
complete -c nala -s d -l download-only -d 'Package files are only retrieved, not unpacked or installed'
complete -c nala -s v -l verbose -d 'Disable scrolling text and print extra information'
complete -c nala -s f -l fix-broken -d 'Attempts to fix broken packages'
complete -c nala -l no-fix-broken -d 'Skips attempting to fix broken packages'
complete -c nala -l no-install-recommends -d 'Stops the installation of recommended packages'
complete -c nala -l no-update -d 'Skips updating the package list'
complete -c nala -l no-autoremove -d 'Stops nala from autoremoving packages'
complete -c nala -l remove-essential -d 'Allows the removal of essential packages'
complete -c nala -l raw-dpkg -d 'Skips all formatting and you get raw dpkg output'
complete -c nala -l update -d 'Updates the package list'
complete -c nala -l debug -d 'Logs extra information for debugging'
complete -c nala -l license -d 'Reads the licenses of software compiled in and then reads the GPLv3'
complete -c nala -l version -d 'Show program\'s version number and exit'
complete -c nala -l no-aptlist -d 'Sets `APT_LISTCHANGES_FRONTEND=none`, apt-listchanges will not bug you'
complete -c nala -l non-interactive -d 'Sets `DEBIAN_FRONTEND=noninteractive`, this also disables apt-listchanges'
complete -c nala -l non-interactive-full -d 'An alias for --non-interactive --confdef --confold'
complete -c nala -l confold -d 'Always keep the old version without prompting'
complete -c nala -l confnew -d 'Always install the new version without prompting'
complete -c nala -l confdef -d 'Always choose the default action without prompting'
complete -c nala -l confmiss -d 'Always install the missing conffile without prompting. This is dangerous!'
complete -c nala -l confask -d 'Always offer to replace it with the version in the package'

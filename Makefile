enter:
	echo "The Nala MakeFile"

install:
	sudo apt install python3-debian
	sudo python3 -m pip install .

	# Install man pages
	sudo ./nala_build.py man --install

	# Install translations
	sudo python3 -m pip install babel
	sudo ./nala_build.py babel --compile --install

	make completions
	make config

completions:
	# Create Shell Completion Directories

	mkdir -p /usr/share/fish/vendor_completions.d/
	mkdir -p /usr/share/bash-completion/completions/
	mkdir -p /usr/share/zsh/vendor-completions/

	# Install shell completions

	sudo cp debian/nala.fish /usr/share/fish/vendor_completions.d/
	sudo cp debian/bash-completion /usr/share/bash-completion/completions/nala
	sudo cp debian/_nala /usr/share/zsh/vendor-completions/

config:
	# Install the Nala Configuration file.
	sudo mkdir -p /etc/nala
	sudo cp debian/nala.conf /etc/nala/nala.conf

clean:
	rm -f docs/nala*.8
	rm -rf ./.venv

uninstall:
	sudo rm -f /usr/share/man/man8/nala*8.gz
	sudo rm -rf /etc/nala
	sudo python3 -m pip uninstall nala

binary:
	./nala-pyinstall.sh

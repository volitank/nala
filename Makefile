PYTHON_VER="3.10.5"
LIB_DIR="/usr/lib/python3/dist-packages"

enter:
	echo "The Nala MakeFile"

install:
	sudo python3 -m pip install .
	sudo apt install python3-debian

	# Install man pages
	sudo ./nala_build.py man --install

	# Install translations
	sudo python3 -m pip install babel
	sudo ./nala_build.py babel --compile --install

	make completions
	make config

legacy:
	make python
	sudo python3.10 -m pip install .
	sudo apt install python3-debian
	make link-libs

	# Install man pages
	sudo python3.10 nala_build.py man --install

	# Install translations
	sudo python3.10 -m pip install babel
	sudo python3.10 ./nala_build.py babel --compile --install

	make completions
	make config

legacy-update:
	sudo python3.10 -m pip install .
	sudo apt install python3-debian
	make link-libs

	# Install man pages
	sudo python3.10 nala_build.py man --install

	# Install translations
	sudo python3.10 -m pip install babel
	sudo python3.10 ./nala_build.py babel --compile --install

	sudo python3.10 -m pip install babel

	make completions
	make config

python:
	# Download Python
	wget https://www.python.org/ftp/python/${PYTHON_VER}/Python-${PYTHON_VER}.tar.xz

	# Extract Python
	tar -xvf Python-${PYTHON_VER}.tar.xz

	# Change into the python directory
	# Configure Python
	# Build Python with the max threads of the host
	# Alt Install won't replace the default python3
	cd ./Python-${PYTHON_VER}/ && \
		./configure --enable-optimizations && \
		make -j $(nproc) && \
		sudo make altinstall

link-libs:
	# Link python3-apt libraries
	sudo ln -sf $(shell find ${LIB_DIR}/ -name apt_pkg.cpython*.so) ${LIB_DIR}/apt_pkg.so
	sudo ln -sf $(shell find ${LIB_DIR}/ -name apt_inst.cpython*.so) ${LIB_DIR}/apt_inst.so

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

uninstall-legacy:
	sudo rm -f /usr/share/man/man8/nala*8.gz
	sudo rm -rf /etc/nala
	sudo rm -f ${LIB_DIR}/apt_pkg.so
	sudo rm -f ${LIB_DIR}/apt_inst.so
	sudo python3.10 -m pip uninstall nala

binary:
	./nala-pyinstall.sh

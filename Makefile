PREFIX ?= /usr
DATADIR ?= $(PREFIX)/share
MANDIR ?= $(DATADIR)/man
LOCALEDIR ?= $(DATADIR)/locale
SYSCONFDIR ?= /etc
# this location varies, but this is the default for debian
ZSH_COMPLETIONS_DIR ?= $(DATADIR)/zsh/vendor-completions


enter:
	@echo "The Nala makefile"
	@echo ""
	@echo "some commands may need to be run as root (sudo)"
	@echo ""
	@echo "make install"
	@echo "    to install nala"
	@echo "make uninstall"
	@echo "    to uninstall nala"

install: install-data
	apt install python3-debian
	pip install .

install-data: completions translations config manpages

manpages:
	install -d $(DESTDIR)$(MANDIR)/man8
	./scripts/doc build $(DESTDIR)$(MANDIR)/man8

completions:
	install -Dm 644 debian/nala.fish $(DESTDIR)$(DATADIR)/fish/vendor_completions.d/nala.fish
	install -Dm 644 debian/bash-completion $(DESTDIR)$(DATADIR)/bash-completion/completions/nala
	install -Dm 644 debian/_nala $(DESTDIR)$(ZSH_COMPLETIONS_DIR)/_nala

config:
	install -Dm 644 debian/nala.conf $(DESTDIR)$(SYSCONFDIR)/nala/nala.conf

translations:
	./scripts/translations compile $(DESTDIR)$(LOCALEDIR)

update-translations:
	./scripts/translations update

clean:
	-rm -f docs/nala*.8
	-rm -rf ./.venv
	-rm -rf ./out

uninstall-data:
	-rm -rf $(DESTDIR)$(SYSCONFDIR)/nala
	-rm -f $(DESTDIR)$(MANDIR)/man8/nala*8*
	-rm -f $(DESTDIR)$(LOCALEDIR)/*/LC_MESSAGES/nala.mo
	-rm -f $(DESTDIR)$(DATADIR)/bash-completion/completions/nala
	-rm -f $(DESTDIR)$(DATADIR)/fish/vendor_completions.d/nala.fish
	-rm -f $(DESTDIR)$(ZSH_COMPLETIONS_DIR)/_nala

uninstall: uninstall-data
	python3 -m pip uninstall nala

binary:
	./nala-pyinstall.sh

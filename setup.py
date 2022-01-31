# Always prefer setuptools over distutils
"""Setup.py."""
from pathlib import Path

from setuptools import setup

import nala

# Define the directory that setup.py is in
here = Path(__file__).parent.resolve()

# Get the long description from the README file
long_description = (here / 'README.rst').read_text(encoding='utf-8')

# Arguments marked as "Required" below must be included for upload to PyPI.
# Fields marked as "Optional" may be commented out.

setup(
	name='nala',  # Required
	version=nala.__version__,  # Required
	description='a wrapper for the apt package manager.',  # Optional
	long_description=long_description,  # Optional
	long_description_content_type='text/reStructuredText',  # Optional (see note above)
	url='https://gitlab.com/volian/nala',  # Optional
	author='Blake Lee (volitank)',  # Optional
	author_email='blake@volitank.com',  # Optional
	classifiers=[  # Optional
	# List of classifiers https://gist.github.com/nazrulworld/3800c84e28dc464b2b30cec8bc1287fc
		'Development Status :: 3 - Alpha',
		'Environment :: Console',
		'Intended Audience :: End Users/Desktop',
		'License :: OSI Approved :: GNU General Public License v3 or later (GPLv3+)',
		'Natural Language :: English',
		'Operating System :: POSIX :: Linux',
		'Topic :: System :: Operating System Kernels :: Linux',
		'Programming Language :: Python :: 3',
		'Programming Language :: Python :: 3.8',
		'Programming Language :: Python :: 3.9',
		'Programming Language :: Python :: 3 :: Only',
	],

	keywords='nala, package management, apt',  # Optional
	packages=['nala'],  # Required
	python_requires='>=3.8, <4',
	install_requires=[
		'pexpect',
		'aiofiles',
		'jsbeautifier',
		'pyyaml',
		'pythonping==1.1.1',
		'rich==11.1.0',
		'httpx==0.22.0',
	],

	entry_points={  # Optional
		'console_scripts': [
			'nala=nala.__main__:main',
		],
	},

	project_urls={  # Optional
		'Documentation': 'https://gitlab.com/volian/nala',
		'Source': 'https://gitlab.com/volian/nala',
	},
)

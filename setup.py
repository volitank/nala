# Always prefer setuptools over distutils
from setuptools import setup
from pathlib import Path
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
	url='https://salsa.debian.org/volian-team/nala',  # Optional
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
		'Programming Language :: Python :: 3.6',
		'Programming Language :: Python :: 3.7',
		'Programming Language :: Python :: 3.8',
		'Programming Language :: Python :: 3.9',
		'Programming Language :: Python :: 3 :: Only',
	],

	keywords='nala, package management, apt',  # Optional
	packages=['nala'],  # Required
	package_data={  # Optional
		'nala': ['LICENSE'],
	},
	python_requires='>=3.6, <4',
	entry_points={  # Optional
		'console_scripts': [
			'nala=nala.__main__:main',
		],
	},

	project_urls={  # Optional
		'Documentation': 'https://salsa.debian.org/volian-team/nala',
		'Source': 'https://salsa.debian.org/volian-team/nala',
	},
)
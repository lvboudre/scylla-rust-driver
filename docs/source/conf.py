# -*- coding: utf-8 -*-
import os
import sys
from datetime import date

from recommonmark.transform import AutoStructify
from pygments.lexers.configs import TOMLLexer
from pygments.lexers.rust import RustLexer
from sphinx.highlighting import lexers

from sphinx_scylladb_theme.utils import multiversion_regex_builder
sys.path.insert(0, os.path.abspath('..'))

# -- Global variables

# Build documentation for the following tags and branches
TAGS = ['v0.10.1', 'v0.11.1']
BRANCHES = ['main']
# Set the latest version.
LATEST_VERSION = 'v0.11.1'
# Set which versions are not released yet.
UNSTABLE_VERSIONS = ['main']
# Set which versions are deprecated
DEPRECATED_VERSIONS = ['v0.10.1']

# -- General configuration

# Add any Sphinx extension module names here, as strings. They can be
# extensions coming with Sphinx (named 'sphinx.ext.*') or your custom
# ones.
extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.todo',
    'sphinx.ext.mathjax',
    'sphinx.ext.githubpages',
    'sphinx.ext.extlinks',
    'sphinx_sitemap',
    'sphinx_scylladb_theme',
    'sphinx_multiversion',  # optional
    'recommonmark',  # optional
]

# The suffix(es) of source filenames.
# You can specify multiple suffix as a list of string:
#
source_suffix = ['.rst', '.md']
autosectionlabel_prefix_document = True

# The master toctree document.
master_doc = 'contents'

# General information about the project.
project = 'Scylla Rust Driver'
copyright = str(date.today().year) + ', ScyllaDB. All rights reserved.'
author = u'Scylla Project Contributors'

# List of patterns, relative to source directory, that match files and
# directories to ignore when looking for source files.
# This patterns also effect to html_static_path and html_extra_path
exclude_patterns = ['_build', 'Thumbs.db', '.DS_Store', '_utils', 'SUMMARY.md']

# The name of the Pygments (syntax highlighting) style to use.
pygments_style = 'sphinx'

# Setup Sphinx
def setup(sphinx):
    sphinx.add_config_value('recommonmark_config', {
        'enable_eval_rst': True,
        'enable_auto_toc_tree': False,
    }, True)
    sphinx.add_transform(AutoStructify)
    lexers['rust'] = RustLexer()
    lexers['toml'] = TOMLLexer()


# -- Options for not found extension

# Template used to render the 404.html generated by this extension.
notfound_template =  '404.html'

# Prefix added to all the URLs generated in the 404 page.
notfound_urls_prefix = ''

# -- Options for multiversion extension

# Whitelist pattern for tags (set to None to ignore all tags)
smv_tag_whitelist = multiversion_regex_builder(TAGS)
# Whitelist pattern for branches
smv_branch_whitelist = multiversion_regex_builder(BRANCHES)
# Defines which version is considered to be the latest stable version.
smv_latest_version = LATEST_VERSION
smv_rename_latest_version = 'stable'
# Whitelist pattern for remotes (set to None to use local branches only)
smv_remote_whitelist = r'^origin$'
# Pattern for released versions
smv_released_pattern = r'^tags/.*$'
# Format for versioned output directories inside the build directory
smv_outputdir_format = '{ref.name}'

# -- Options for sitemap extension

sitemap_url_scheme = "/stable/{link}"

# -- Options for HTML output

# The theme to use for HTML and HTML Help pages.  See the documentation for
# a list of builtin themes.
#
html_theme = 'sphinx_scylladb_theme'

# Theme options are theme-specific and customize the look and feel of a theme
# further.  For a list of options available for each theme, see the
# documentation.
html_theme_options = {
    'conf_py_path': 'docs/source/',
    'default_branch': 'main',
    'github_repository': 'scylladb/scylla-rust-driver',
    'github_issues_repository': 'scylladb/scylla-rust-driver',
    'hide_banner': 'true',
    'site_description': 'Scylla Driver for Rust.',
    'hide_edit_this_page_button': 'false',
    'hide_feedback_buttons': 'false',
    'versions_unstable': UNSTABLE_VERSIONS,
    'versions_deprecated': DEPRECATED_VERSIONS,
}

# If not None, a 'Last updated on:' timestamp is inserted at every page
# bottom, using the given strftime format.
# The empty string is equivalent to '%b %d, %Y'.
html_last_updated_fmt = '%d %B %Y'

# Custom sidebar templates, maps document names to template names.
#
html_sidebars = {'**': ['side-nav.html']}

# Output file base name for HTML help builder.
htmlhelp_basename = 'ScyllaDocumentationdoc'

# URL which points to the root of the HTML documentation. 
html_baseurl = 'https://rust-driver.docs.scylladb.com'

# Dictionary of values to pass into the template engine’s context for all pages
html_context = {'html_baseurl': html_baseurl}
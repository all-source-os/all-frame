#!/usr/bin/env python3
"""
Generate changelog.html from CHANGELOG.md for GitHub Pages.
Usage: python3 scripts/generate_changelog_html.py CHANGELOG.md > docs/site/changelog.html
"""

import re
import sys
from datetime import datetime


def parse_changelog(content):
    """Parse CHANGELOG.md and extract releases."""
    releases = []
    current_release = None
    current_section = None

    lines = content.split('\n')
    i = 0
    while i < len(lines):
        line = lines[i]

        # Match release header: ## [0.1.9] - 2025-12-13
        release_match = re.match(r'^## \[([^\]]+)\] - (\d{4}-\d{2}-\d{2})', line)
        if release_match:
            if current_release:
                releases.append(current_release)
            current_release = {
                'version': release_match.group(1),
                'date': release_match.group(2),
                'title': '',
                'tags': [],
                'sections': {}
            }
            current_section = None
            i += 1
            continue

        # Match section header: ### Added, ### Changed, ### Fixed
        section_match = re.match(r'^### (.+)$', line)
        if section_match and current_release:
            current_section = section_match.group(1).strip()
            current_release['sections'][current_section] = []
            i += 1
            continue

        # Match list items
        if line.startswith('- ') and current_release and current_section:
            item = line[2:].strip()
            current_release['sections'][current_section].append(item)

        i += 1

    if current_release:
        releases.append(current_release)

    return releases


def format_date(date_str):
    """Format date string to human readable."""
    try:
        dt = datetime.strptime(date_str, '%Y-%m-%d')
        return dt.strftime('%B %d, %Y')
    except:
        return date_str


def get_tags(release):
    """Determine tags based on content."""
    tags = []
    sections = release.get('sections', {})

    if 'Added' in sections:
        tags.append(('new', 'New'))
    if 'Changed' in sections:
        tags.append(('feature', 'Changed'))
    if 'Fixed' in sections:
        tags.append(('fix', 'Fixed'))
    if 'Breaking' in sections or 'BREAKING' in str(sections):
        tags.append(('breaking', 'Breaking'))

    # Add feature tags based on content
    content = str(sections).lower()
    if 'auth' in content:
        tags.append(('feature', 'Authentication'))
    elif 'resilience' in content or 'circuit' in content or 'retry' in content:
        tags.append(('feature', 'Resilience'))
    elif 'cqrs' in content or 'event' in content:
        tags.append(('feature', 'CQRS'))
    elif 'mcp' in content:
        tags.append(('feature', 'MCP'))
    elif 'grpc' in content or 'graphql' in content or 'protocol' in content:
        tags.append(('feature', 'Multi-Protocol'))
    elif 'shutdown' in content:
        tags.append(('feature', 'Production-Ready'))
    elif 'security' in content:
        tags.append(('feature', 'Security'))

    return tags[:4]  # Limit to 4 tags


def get_title(release):
    """Generate a title from the release content."""
    sections = release.get('sections', {})
    added = sections.get('Added', [])

    if added:
        # Get first bold item
        for item in added:
            match = re.match(r'\*\*([^*]+)\*\*', item)
            if match:
                return match.group(1)
        return added[0][:50] + '...' if len(added[0]) > 50 else added[0]

    return f"Version {release['version']}"


def escape_html(text):
    """Escape HTML special characters."""
    return text.replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')


def format_item(item):
    """Format a changelog item to HTML."""
    # Convert **bold** to <strong>
    item = re.sub(r'\*\*([^*]+)\*\*', r'<strong>\1</strong>', item)
    # Convert `code` to <code class="code-inline">
    item = re.sub(r'`([^`]+)`', r'<code class="code-inline">\1</code>', item)
    # Convert [links](url) to <a>
    item = re.sub(r'\[([^\]]+)\]\(([^)]+)\)', r'<a href="\2">\1</a>', item)
    return item


def generate_release_html(release):
    """Generate HTML for a single release."""
    version = release['version']
    date = format_date(release['date'])
    title = get_title(release)
    tags = get_tags(release)
    sections = release['sections']

    tags_html = '\n                            '.join([
        f'<span class="tag {t[0]}">{t[1]}</span>' for t in tags
    ])

    sections_html = ''
    for section_name, items in sections.items():
        if not items:
            continue
        items_html = '\n                                '.join([
            f'<li>{format_item(item)}</li>' for item in items[:10]  # Limit items
        ])
        sections_html += f'''
                        <div class="section">
                            <h3 class="section-title">{escape_html(section_name)}</h3>
                            <ul>
                                {items_html}
                            </ul>
                        </div>
'''

    return f'''
                <!-- v{version} -->
                <div class="release">
                    <div class="release-meta">
                        <div class="release-date">{date}</div>
                        <div class="release-version">{version}</div>
                    </div>
                    <div class="release-content">
                        <h2 class="release-title">{escape_html(title)}</h2>
                        <div class="release-tags">
                            {tags_html}
                        </div>
{sections_html}
                    </div>
                </div>
'''


HTML_TEMPLATE = '''<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Changelog - AllFrame</title>
    <meta name="description" content="AllFrame changelog - Track all releases, features, and improvements to the composable Rust API framework.">
    <meta name="keywords" content="rust, web framework, changelog, releases, allframe">
    <link rel="canonical" href="https://all-source-os.github.io/all-frame/changelog.html">

    <!-- Open Graph -->
    <meta property="og:title" content="Changelog - AllFrame">
    <meta property="og:description" content="Track all releases, features, and improvements to AllFrame.">
    <meta property="og:type" content="website">
    <meta property="og:url" content="https://all-source-os.github.io/all-frame/changelog.html">

    <style>
        :root {
            --primary: #f74c00;
            --primary-dark: #d94400;
            --bg: #0d1117;
            --bg-secondary: #161b22;
            --bg-tertiary: #21262d;
            --text: #e6edf3;
            --text-muted: #8b949e;
            --border: #30363d;
            --green: #3fb950;
            --blue: #58a6ff;
            --purple: #a371f7;
            --yellow: #d29922;
        }

        * { margin: 0; padding: 0; box-sizing: border-box; }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans', Helvetica, Arial, sans-serif;
            background: var(--bg);
            color: var(--text);
            line-height: 1.6;
        }

        a { color: var(--blue); text-decoration: none; }
        a:hover { text-decoration: underline; }

        .container { max-width: 1000px; margin: 0 auto; padding: 0 24px; }

        header {
            border-bottom: 1px solid var(--border);
            padding: 16px 0;
            position: sticky;
            top: 0;
            background: var(--bg);
            z-index: 100;
        }

        nav { display: flex; justify-content: space-between; align-items: center; }

        .logo {
            font-size: 1.5rem;
            font-weight: 700;
            color: var(--text);
            display: flex;
            align-items: center;
            gap: 8px;
        }

        .logo span { color: var(--primary); }

        .nav-links { display: flex; gap: 24px; align-items: center; }
        .nav-links a { color: var(--text-muted); font-size: 0.95rem; }
        .nav-links a:hover { color: var(--text); text-decoration: none; }

        .page-header { padding: 48px 0 32px; border-bottom: 1px solid var(--border); }
        .page-header h1 { font-size: 2.5rem; font-weight: 700; margin-bottom: 8px; }
        .page-header p { color: var(--text-muted); font-size: 1.1rem; }

        .timeline { padding: 40px 0; }

        .release {
            display: flex;
            gap: 32px;
            position: relative;
            padding-bottom: 48px;
        }

        .release:last-child { padding-bottom: 0; }

        .release-meta {
            width: 140px;
            flex-shrink: 0;
            position: sticky;
            top: 100px;
            align-self: flex-start;
        }

        .release-date { font-size: 0.9rem; color: var(--text-muted); margin-bottom: 12px; }

        .release-version {
            display: inline-flex;
            align-items: center;
            justify-content: center;
            width: 56px;
            height: 40px;
            background: var(--bg-secondary);
            border: 1px solid var(--border);
            border-radius: 8px;
            font-weight: 700;
            font-size: 0.9rem;
        }

        .release-content {
            flex: 1;
            position: relative;
            padding-left: 32px;
        }

        .release-content::before {
            content: '';
            position: absolute;
            left: 0;
            top: 8px;
            bottom: -48px;
            width: 1px;
            background: var(--border);
        }

        .release:last-child .release-content::before { bottom: 0; }

        .release-content::after {
            content: '';
            position: absolute;
            left: -5px;
            top: 8px;
            width: 11px;
            height: 11px;
            background: var(--primary);
            border-radius: 50%;
        }

        .release-title { font-size: 1.5rem; font-weight: 600; margin-bottom: 12px; }

        .release-tags { display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 20px; }

        .tag {
            display: inline-flex;
            align-items: center;
            padding: 4px 12px;
            background: var(--bg-tertiary);
            border: 1px solid var(--border);
            border-radius: 20px;
            font-size: 0.8rem;
            color: var(--text-muted);
        }

        .tag.new { background: rgba(63, 185, 80, 0.15); border-color: var(--green); color: var(--green); }
        .tag.breaking { background: rgba(247, 76, 0, 0.15); border-color: var(--primary); color: var(--primary); }
        .tag.feature { background: rgba(88, 166, 255, 0.15); border-color: var(--blue); color: var(--blue); }
        .tag.fix { background: rgba(163, 113, 247, 0.15); border-color: var(--purple); color: var(--purple); }

        .section { margin-bottom: 24px; }

        .section-title {
            font-size: 1rem;
            font-weight: 600;
            color: var(--text-muted);
            margin-bottom: 12px;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }

        .section ul { list-style: none; padding: 0; }

        .section li {
            position: relative;
            padding-left: 20px;
            margin-bottom: 8px;
            color: var(--text);
        }

        .section li::before {
            content: '';
            position: absolute;
            left: 0;
            top: 10px;
            width: 6px;
            height: 6px;
            background: var(--text-muted);
            border-radius: 50%;
        }

        .section li strong { color: var(--text); }

        .code-inline {
            background: var(--bg-tertiary);
            padding: 2px 6px;
            border-radius: 4px;
            font-family: 'SF Mono', 'Fira Code', monospace;
            font-size: 0.85em;
        }

        footer {
            padding: 40px 0;
            border-top: 1px solid var(--border);
            text-align: center;
            color: var(--text-muted);
            font-size: 0.9rem;
        }

        footer .links { display: flex; gap: 24px; justify-content: center; margin-bottom: 16px; }

        @media (max-width: 768px) {
            .release { flex-direction: column; gap: 16px; }
            .release-meta { width: 100%; position: static; display: flex; align-items: center; gap: 16px; }
            .release-content { padding-left: 0; }
            .release-content::before, .release-content::after { display: none; }
            .nav-links { display: none; }
        }
    </style>
</head>
<body>
    <header>
        <nav class="container">
            <a href="index.html" class="logo">
                <span>All</span>Frame
            </a>
            <div class="nav-links">
                <a href="index.html">Home</a>
                <a href="https://docs.rs/allframe">Docs</a>
                <a href="https://github.com/all-source-os/all-frame">GitHub</a>
                <a href="https://crates.io/crates/allframe">crates.io</a>
            </div>
        </nav>
    </header>

    <main>
        <div class="page-header">
            <div class="container">
                <h1>Changelog</h1>
                <p>Track all releases, features, and improvements to AllFrame.</p>
            </div>
        </div>

        <div class="timeline">
            <div class="container">
{releases_html}
            </div>
        </div>
    </main>

    <footer>
        <div class="container">
            <div class="links">
                <a href="index.html">Home</a>
                <a href="https://github.com/all-source-os/all-frame">GitHub</a>
                <a href="https://docs.rs/allframe">Docs</a>
                <a href="https://crates.io/crates/allframe">crates.io</a>
            </div>
            <p>AllFrame - One frame. Infinite transformations. Built with TDD, from day zero.</p>
        </div>
    </footer>
</body>
</html>
'''


def generate_html(releases):
    """Generate the full changelog HTML page."""
    releases_html = ''.join([generate_release_html(r) for r in releases[:15]])  # Limit to 15 releases
    return HTML_TEMPLATE.replace('{releases_html}', releases_html)


if __name__ == '__main__':
    changelog_path = sys.argv[1] if len(sys.argv) > 1 else 'CHANGELOG.md'

    with open(changelog_path, 'r') as f:
        content = f.read()

    releases = parse_changelog(content)
    html = generate_html(releases)
    print(html)

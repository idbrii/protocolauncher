# protocolauncher

A demonstration of how to create a program that registers itself as a scheme
handler (viewsvn://) and parses urls passed when handling that scheme. To adapt
it to another purpose, change `HANDLED_PROTOCOL` and change parameters to
`Command::new`.


A simple Rust application to launch TortoiseSVN for viewsvn:// urls. Run
without arguments as admin to register. Launch a url [like
this](viewsvn://view?revision=60000&server_url=https://svn.corp.ca/svn/corp_repository):

    viewsvn://view?revision=60000&server_url=https://svn.corp.ca/svn/corp_repository


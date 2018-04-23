# Feeds to Instapaper

Loads Atom or RSS feeds from the provided link list and interactively adds new posts to Instapaper.

It asks user input for every new entry and saves this choice for future use in subsequent runs.

Configuration is provided using environment variables (tool can .env file automatically):

    INSTAPAPER_USERNAME
    INSTAPAPER_PASSWORD
    LINKS_LIST_FILE # put feed urls here (one per line)
    LINKS_LOG_FILE  # processed post urls are stored here

Above files must already exist (for now).

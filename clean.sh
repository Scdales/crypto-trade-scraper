find data/postgres/db-data/ -mindepth 1 ! -name ".gitkeep" -exec rm -rf {} +
find data/redis/cache/ -mindepth 1 ! -name ".gitkeep" -exec rm -rf {} +
find nginx/cert/ -mindepth 1 ! -name ".gitkeep" -exec rm -rf {} +
rm -rf nginx/logs/*
rm -rf src/scrapers/rust*/target
docker volume rm crypto-trade-scraper_grafana_storage crypto-trade-scraper_prometheus_storage crypto-trade-scraper_loki_storage

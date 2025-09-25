set PHP_FCGI_MAX_REQUESTS=0
set PHP_FCGI_CHILDREN=16
php-cgi.exe -b 127.0.0.1:9000 -f index.php
# Meilisearch connector

Synchronize your datasource (currently support PostgresSQL) with Meilisearch instance.

## PostgresSQL:
**Requirement:**
<ul>
    <li>Support Postgres version: above version 9.4</li>
    <li>
    Plugin wal2json must be installed associate with Postgres version on your local machine:

#### In Red Hat/CentOS:

```
$ sudo yum install wal2json14
```

#### In Debian/Ubuntu:

```
$ sudo apt-get install postgresql-14-wal2json
```

You can also keep up with the latest fixes and features cloning the Git repository.

```
$ git clone https://github.com/eulerto/wal2json.git
```

#### Unix based Operating Systems


Before installing **wal2json**, you should have PostgreSQL 9.4+ installed (including the header files). If PostgreSQL is not in your search path, add it. If you are using [PostgreSQL yum repository](https://yum.postgresql.org), install `postgresql14-devel` and add `/usr/pgsql-14/bin` to your search path (yum uses `14, 13, 12, 11, 10, 96 or 95`). If you are using [PostgreSQL apt repository](https://wiki.postgresql.org/wiki/Apt), install `postgresql-server-dev-14` and add `/usr/lib/postgresql/14/bin` to your search path. (apt uses `14, 13, 12, 11, 10, 9.6 or 9.5`).

If you compile PostgreSQL by yourself and install it in `/home/euler/pg14`:

```
$ tar -zxf wal2json-wal2json_2_5.tar.gz
$ cd wal2json-wal2json_2_5
$ export PATH=/home/euler/pg14/bin:$PATH
$ make
$ make install
```
</li>
    <li> Make sure to config Postgres instance with: </li>

```
wal_level = logical
#
# these parameters only need to set in versions 9.4, 9.5 and 9.6
# default values are ok in version 10 or later
#

# each table config to sync with Meilisearch required 1 replication slot.
max_replication_slots = 10
max_wal_senders = 10
```

</ul>

## Configuration

```yaml
meilisearch:
  api_url: http://localhost:7700
  admin_api_key: "4d210c85a27082683758d083d6c1298ee7e1e97385edeaad3250b9cd8aebd276"
  sync_interval_time: 10
  upload_size: 2
data_source:
  source_type: PostgresSQL
  host: localhost
  port: 5432
  username: dattran
  password: dattran
  database: postgres
synchronize_tables:
  - index_name: users
    primary_key: user_id
    sync_fields:
      - user_id
      - username
      - email
      - created_at
```



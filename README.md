```
$$\      $$\           $$\ $$\ $$\                                                   $$\                                                                             $$\
$$$\    $$$ |          \__|$$ |\__|                                                  $$ |                                                                            $$ |
$$$$\  $$$$ | $$$$$$\  $$\ $$ |$$\  $$$$$$$\  $$$$$$\   $$$$$$\   $$$$$$\   $$$$$$$\ $$$$$$$\           $$$$$$$\  $$$$$$\  $$$$$$$\  $$$$$$$\   $$$$$$\   $$$$$$$\ $$$$$$\    $$$$$$\   $$$$$$\
$$\$$\$$ $$ |$$  __$$\ $$ |$$ |$$ |$$  _____|$$  __$$\  \____$$\ $$  __$$\ $$  _____|$$  __$$\ $$$$$$\ $$  _____|$$  __$$\ $$  __$$\ $$  __$$\ $$  __$$\ $$  _____|\_$$  _|  $$  __$$\ $$  __$$\
$$ \$$$  $$ |$$$$$$$$ |$$ |$$ |$$ |\$$$$$$\  $$$$$$$$ | $$$$$$$ |$$ |  \__|$$ /      $$ |  $$ |\______|$$ /      $$ /  $$ |$$ |  $$ |$$ |  $$ |$$$$$$$$ |$$ /        $$ |    $$$$$$$$ |$$ |  \__|
$$ |\$  /$$ |$$   ____|$$ |$$ |$$ | \____$$\ $$   ____|$$  __$$ |$$ |      $$ |      $$ |  $$ |        $$ |      $$ |  $$ |$$ |  $$ |$$ |  $$ |$$   ____|$$ |        $$ |$$\ $$   ____|$$ |
$$ | \_/ $$ |\$$$$$$$\ $$ |$$ |$$ |$$$$$$$  |\$$$$$$$\ \$$$$$$$ |$$ |      \$$$$$$$\ $$ |  $$ |        \$$$$$$$\ \$$$$$$  |$$ |  $$ |$$ |  $$ |\$$$$$$$\ \$$$$$$$\   \$$$$  |\$$$$$$$\ $$ |
\__|     \__| \_______|\__|\__|\__|\_______/  \_______| \_______|\__|       \_______|\__|  \__|         \_______| \______/ \__|  \__|\__|  \__| \_______| \_______|   \____/  \_______|\__|

                                                                                        Version: 0.1
                                                                                        Author: dattd
```

Synchronize your datasource (currently support PostgresSQL) with Meilisearch instance.

## PostgresSQL:
**Requirement:**
<ul>
    <li><strong>Support Postgres version: above version 12</strong></li>
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

#### In MacOS:

```
$ brew install wal2json
```
</li>
    <li> Make sure to config Postgres wal_level and have enough replication slot: </li>

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

## Meilisearch:
Make sure to cover core concepts of Meilisearch such as index, document, index policy and more at: https://www.meilisearch.com/docs

## Configuration

```yaml
meilisearch:
  api_url: http://localhost:7700
  admin_api_key: "4d210c85a27082683758d083d6c1298ee7e1e97385edeaad3250b9cd8aebd276"
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
      - created_at
```
<strong>* Note: This is a POC and still in development process. Base source can change drastically in the future, please consider to use in production environment with real data.</strong>




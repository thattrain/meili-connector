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
# Features:
<ol>
    <li>Synchronize data from your data source to Meilisearch.</li>
    <li>Perform change data capture (CDC) to remain data consistency.</li>
    <li>Refresh data in case of inconsistency between your data source and Meilisearch instance using Meilisearch's swap index feature (performing cost no downtime).</li>
    <li>Show consistent status between your datasource and Meilisearch instance (record wise).</li>
    <li>Config search policy of each index. For more documentation please refer to official Meilisearch documentation <a href="https://www.meilisearch.com/docs/reference/api/settings">here</a> .</li>

</ol>


Synchronize your datasource (currently support PostgresSQL) with Meilisearch instance.

## PostgresSQL:
**Requirement:**
<ul>
    <li><strong>Support Postgres version: above version 12</strong></li>
    <li>
    Plugin wal2json must be installed associate with Postgres version on your local machine:

#### In Red Hat/CentOS:

```bash
sudo yum install wal2json14
```

#### In Debian/Ubuntu:

```bash
sudo apt-get install postgresql-14-wal2json
```

#### In MacOS:

```bash
brew install wal2json
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
Make sure to cover core concepts of Meilisearch such as index, document, search policy and more at: https://www.meilisearch.com/docs

## How to build

```bash
cargo build --release
```

## How to run
<ul>
<li> Show help

</li>

```bash
cargo run -- -h
```

<li> Sync data and perform CDC

```bash
cargo run -- --config <path> sync
```

</li>

<li> Check for consistency

```bash
cargo run -- --config <path> status
```

</li>

<li> Refresh data by swap index

```bash
cargo run -- --config <path> refresh
```

</li>

</ul>

## Configuration file example

```yaml
meilisearch:
  apiUrl: http://localhost:7700
  adminApiKey: "4d210c85a27082683758d083d6c1298ee7e1e97385edeaad3250b9cd8aebd276"
dataSource:
  sourceType: Postgres
  host: localhost
  port: 5432
  username: dattran
  password: dattran
  database: postgres
synchronizeTables:
  - indexName: users
    primaryKey: user_id
    limit: 1000
    syncFields:
      - user_id
      - username
      - created_at
    meiliSetting:
      displayedAttributes:
        - user_id
        - username
      searchableAttributes:
        - username
  - indexName: employees
    primaryKey: employee_id
    limit: 10
    syncFields:
      - employee_id
      - first_name

```
<strong>* Note: This is a POC and still in development process. Base source can change drastically in the future, please consider to use in production environment with real data.</strong>




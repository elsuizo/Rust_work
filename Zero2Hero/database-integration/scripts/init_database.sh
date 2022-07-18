#! /usr/bin/env bash

# NOTE(elsuizo: 2022-07-17): set es para setear una variable en bash...
set -x
set -eo pipefail

# con esto nos aseguramos de que las dependencias estan instaladas
if ! [ -x "$(command -v psql)" ]; then
   echo >&2 "Error: psql is not installed"
   exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
   echo >&2 "Error: sqlx is not installed"
   echo >&2 "Use:"
   echo >&2 "    cargo install sqlx-cli --no-default-features --features native-tls,postgres"
   echo >&2 "to install it."
   exit 1
fi

# check si el usuario ha sido seteado sino poner como user el default que es
# 'postgres'
DB_USER=${POSTGRES_USER:=postgres}
# check si se ha puesto un password sino poner como default el valor 'password'
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
# check si se ha puesto un nombre a la base de datos sino poner como default el
# valor 'newsletter'
DB_NAME="${POSTGRES_DB:=newsletter}"
# check si se ha puesto un nombre al puerto sino poner el valor por default
# '5432'
DB_PORT="${POSTGRES_PORT:=5432}"

# permitimos evadir docker si es que la base de datos postgres ya esta corriendo
if [[ -z "${SKIP_DOCKER}" ]]
then
docker run \
   -e POSTGRES_USER=${DB_USER}\
   -e POSTGRES_PASSWORD=${DB_PASSWORD}\
   -e POSTGRES_DB=${DB_NAME}\
   -p "${DB_PORT}":5432\
   -d postgres\
   postgres -N 1000
fi

# # lanzar postgres usando Docker
# docker run \
#    -e POSTGRES_USER=${DB_USER}\
#    -e POSTGRES_PASSWORD=${DB_PASSWORD}\
#    -e POSTGRES_DB=${DB_NAME}\
#    -p "${DB_PORT}":5432\
#    -d postgres \
#    postgres -N 1000
#    # ^ incrementa el numero de conexiones para propositos de los tests

# hacemos esto porque parece que no siempre el container esta listo para correr
# los comandos entonces con esto esperamos hasta que este listo para recibir
# comandos
export PGPASSWORD="${DB_PASSWORD}"

until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
   >&2 echo "postgres is still unnavailable - sleeping"
   sleep 1
done

>&2 echo "postgres is up and running on port ${DB_PORT}! ---> running migrations now!"

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run

>&2 echo "postgres has been migrated, ready to go!!!"

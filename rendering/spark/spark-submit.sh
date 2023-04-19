#!/usr/bin/bash

if ! poetry env info; then
    exit 1
elif [[ "$VIRTUAL_ENV" == "" ]]; then
    echo "Error: Poetry env not activated."
    exit 1
fi

export PYSPARK_VENV_FILE=pyspark_venv.tar.gz
export PYSPARK_PYTHON=$(which python3)

venv-pack -o $PYSPARK_VENV_FILE
spark-submit \
    --archives $PYSPARK_VENV_FILE \
    --packages org.apache.hudi:hudi-spark3.3-bundle_2.12:0.13.0 \
    --conf 'spark.serializer=org.apache.spark.serializer.KryoSerializer' \
    --conf 'spark.sql.catalog.spark_catalog=org.apache.spark.sql.hudi.catalog.HoodieCatalog' \
    --conf 'spark.sql.extensions=org.apache.spark.sql.hudi.HoodieSparkSessionExtension' \
    "${@:1}"

#! /usr/bin/env sh

response=`curl -X 'GET' "https://$VAULT_HOST/v1/$VAULT_SECRET_PATH" -s \
  -H 'accept: application/json' \
  -H "X-Vault-Token: $VAULT_TOKEN"`

data=`echo $response | jq -r '.data.data'`

for key in $(echo "$data" | jq -r 'keys[]'); do
    value=$(echo "$data" | jq -r ".\"$key\"")  # Corrected syntax
    echo "$key"="$value"
done

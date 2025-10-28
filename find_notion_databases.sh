#!/usr/bin/env bash

# Script to find databases in Notion
# Usage: ./find_notion_databases.sh

echo "🔍 Searching for databases in your Notion workspace..."
echo

# Check for .env file existence
if [ ! -f ".env" ]; then
    echo "❌ .env file not found!"
    exit 1
fi

# Load variables from .env
set -o allexport
source .env
set +o allexport

# Check for API key presence
if [ -z "$NOTION_API_KEY" ] || [ "$NOTION_API_KEY" = "your_notion_api_key_here" ]; then
    echo "❌ NOTION_API_KEY not configured!"
    exit 1
fi

echo "✅ API key found"
echo

# Search for all databases
echo "📋 Searching for all databases..."
response=$(curl -s -X POST 'https://api.notion.com/v1/search' \
    -H "Authorization: Bearer $NOTION_API_KEY" \
    -H 'Content-Type: application/json' \
    -H 'Notion-Version: 2022-06-28' \
    --data '{
        "filter": {
            "value": "database",
            "property": "object"
        }
    }')

# Check if response contains results
if echo "$response" | jq -e '.results' > /dev/null 2>&1; then
    echo "✅ Found databases:"
    echo
    
    # Show found databases
    echo "$response" | jq -r '.results[] | "📊 \(.title[0].plain_text // "Untitled") - ID: \(.id)"'
    
    echo
    echo "💡 Copy the ID of the desired database to your .env file"
    echo "   Format: NOTION_DATABASE_ID=your_id_here"
    
else
    echo "❌ Error getting database list:"
    echo "$response" | jq -r '.message // "Unknown error"'
    echo
    echo "🔧 Possible causes:"
    echo "   1. Incorrect API key"
    echo "   2. No available databases"
    echo "   3. Integration doesn't have necessary permissions"
fi

echo
echo "📚 See detailed instructions in NOTION_SETUP.md"
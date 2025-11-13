#!/usr/bin/env bash

# Script for testing connection to Notion API
# Usage: ./test_notion_connection.sh

echo "🔍 Testing connection to Notion API..."
echo

# Check for .env file existence
if [ ! -f ".env" ]; then
    echo "❌ .env file not found!"
    echo "📋 Copy env.example to .env and fill in the required data:"
    echo "   cp env.example .env"
    echo "   nano .env"
    exit 1
fi

# Load variables from .env
set -o allexport
source .env
set +o allexport

# Check for API key presence
if [ -z "$NOTION_API_KEY" ] || [ "$NOTION_API_KEY" = "your_notion_api_key_here" ]; then
    echo "❌ NOTION_API_KEY not configured!"
    echo "📝 Get API key from https://www.notion.so/my-integrations"
    exit 1
fi

# Check for Database ID presence
if [ -z "$NOTION_DATABASE_ID" ] || [ "$NOTION_DATABASE_ID" = "your_database_id_here" ]; then
    echo "❌ NOTION_DATABASE_ID not configured!"
    echo "📝 See instructions in NOTION_SETUP.md"
    exit 1
fi

echo "✅ .env file found"
echo "✅ NOTION_API_KEY configured"
echo "✅ NOTION_DATABASE_ID configured"
echo

# Test connection to API
echo "🌐 Testing connection to Notion API..."
response=$(curl -s -o /dev/null -w "%{http_code}" \
    -X GET "https://api.notion.com/v1/databases/$NOTION_DATABASE_ID" \
    -H "Authorization: Bearer $NOTION_API_KEY" \
    -H "Notion-Version: 2022-06-28")

if [ "$response" = "200" ]; then
    echo "✅ Connection to Notion API successful!"
    echo "✅ Database found and accessible"
    echo
    echo "🚀 You can run the application: cargo run"
elif [ "$response" = "404" ]; then
    echo "❌ Database not found (404)"
    echo "🔧 Possible causes:"
    echo "   1. Incorrect Database ID"
    echo "   2. Database not shared with integration"
    echo "   3. Database ID contains dashes (need to remove them)"
elif [ "$response" = "401" ]; then
    echo "❌ Unauthorized access (401)"
    echo "🔧 Check API key correctness"
elif [ "$response" = "403" ]; then
    echo "❌ Access forbidden (403)"
    echo "🔧 Integration doesn't have access to database"
    echo "   Share the database with integration via 'Share'"
else
    echo "❌ Unknown error (code: $response)"
fi

echo
echo "📚 See detailed instructions in NOTION_SETUP.md"
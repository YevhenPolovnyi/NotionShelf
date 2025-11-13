#!/bin/bash

# Check Database Properties Script
# This script helps you identify the property names in your Notion database

set -e

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
else
    echo "❌ .env file not found. Please create it from env.example"
    exit 1
fi

# Check if required variables are set
if [ -z "$NOTION_API_KEY" ] || [ "$NOTION_API_KEY" = "your_notion_api_key_here" ]; then
    echo "❌ NOTION_API_KEY not configured in .env file"
    exit 1
fi

if [ -z "$NOTION_DATABASE_ID" ] || [ "$NOTION_DATABASE_ID" = "your_database_id_here" ]; then
    echo "❌ NOTION_DATABASE_ID not configured in .env file"
    exit 1
fi

echo "🔍 Fetching database properties for database: $NOTION_DATABASE_ID"
echo ""

# Make API call to get database schema
response=$(curl -s -X GET \
  "https://api.notion.com/v1/databases/$NOTION_DATABASE_ID" \
  -H "Authorization: Bearer $NOTION_API_KEY" \
  -H "Notion-Version: 2022-06-28" \
  -H "Content-Type: application/json")

# Check if the request was successful
if echo "$response" | grep -q '"status":'; then
    echo "❌ Error from Notion API:"
    echo "$response" | grep -o '"message":"[^"]*"' | sed 's/"message":"\(.*\)"/\1/'
    exit 1
fi

echo "✅ Database found! Here are the available properties:"
echo ""

# Extract property names and types
echo "$response" | python3 -c "
import json
import sys

data = json.load(sys.stdin)
properties = data.get('properties', {})

print('Property Name'.ljust(25) + 'Type'.ljust(15) + 'Description')
print('-' * 60)

for name, prop in properties.items():
    prop_type = prop.get('type', 'unknown')
    
    # Add description based on type
    descriptions = {
        'title': 'Main title field (required)',
        'rich_text': 'Text content',
        'number': 'Numeric values',
        'select': 'Single choice from list',
        'multi_select': 'Multiple choices from list',
        'date': 'Date values',
        'checkbox': 'True/false values',
        'url': 'Website links',
        'email': 'Email addresses',
        'phone_number': 'Phone numbers',
        'relation': 'Links to other pages',
        'rollup': 'Calculated from relations',
        'formula': 'Calculated values',
        'created_time': 'Page creation time',
        'created_by': 'Page creator',
        'last_edited_time': 'Last edit time',
        'last_edited_by': 'Last editor'
    }
    
    description = descriptions.get(prop_type, 'Other type')
    print(name.ljust(25) + prop_type.ljust(15) + description)

print('')
print('💡 Recommendations:')
print('   - Use a \"title\" type property for book titles')
print('   - Use a \"rich_text\" type property for authors')
print('')
print('📝 Update your .env file with these property names:')
print('   NOTION_TITLE_PROPERTY=<your_title_property_name>')
print('   NOTION_AUTHOR_PROPERTY=<your_author_property_name>')
"

echo ""
echo "✅ Done! Configure these property names in your .env file."
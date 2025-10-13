#!/bin/bash

# Seed test data for benchmarking
# Creates users and tasks for realistic load testing

set -e

BASE_URL="${BASE_URL:-http://localhost:3000}"
NUM_USERS="${NUM_USERS:-10}"
TASKS_PER_USER="${TASKS_PER_USER:-20}"

echo "========================================"
echo "Seeding Test Data for Benchmarks"
echo "========================================"
echo "Base URL: $BASE_URL"
echo "Users to create: $NUM_USERS"
echo "Tasks per user: $TASKS_PER_USER"
echo "========================================"
echo ""

# Array to store tokens
declare -a TOKENS

# Create users and login
echo "Creating users..."
for i in $(seq 1 $NUM_USERS); do
    EMAIL="bench-user-$i@example.com"
    PASSWORD="SecurePass123!"
    
    # Register user
    RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/register" \
        -H "Content-Type: application/json" \
        -d "{
            \"name\": \"Benchmark User $i\",
            \"email\": \"$EMAIL\",
            \"password\": \"$PASSWORD\"
        }")
    
    # Extract token
    TOKEN=$(echo $RESPONSE | grep -o '"token":"[^"]*' | cut -d'"' -f4)
    
    if [ -z "$TOKEN" ]; then
        # User might already exist, try logging in
        RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
            -H "Content-Type: application/json" \
            -d "{
                \"email\": \"$EMAIL\",
                \"password\": \"$PASSWORD\"
            }")
        
        TOKEN=$(echo $RESPONSE | grep -o '"token":"[^"]*' | cut -d'"' -f4)
    fi
    
    if [ -n "$TOKEN" ]; then
        TOKENS[$i]=$TOKEN
        echo "✓ User $i created/logged in"
    else
        echo "✗ Failed to create/login user $i"
        exit 1
    fi
done

echo ""
echo "Creating tasks..."

# Create tasks for each user
for i in $(seq 1 $NUM_USERS); do
    TOKEN=${TOKENS[$i]}
    
    for j in $(seq 1 $TASKS_PER_USER); do
        STATUS=$((RANDOM % 3))
        case $STATUS in
            0) STATUS_NAME="todo" ;;
            1) STATUS_NAME="in_progress" ;;
            2) STATUS_NAME="done" ;;
        esac
        
        curl -s -X POST "$BASE_URL/api/v1/tasks" \
            -H "Content-Type: application/json" \
            -H "Authorization: Bearer $TOKEN" \
            -d "{
                \"title\": \"Benchmark Task $j for User $i\",
                \"description\": \"This is a test task created for benchmarking purposes. Status: $STATUS_NAME\",
                \"status\": \"$STATUS_NAME\"
            }" > /dev/null
        
        # Show progress
        if [ $((j % 5)) -eq 0 ]; then
            echo "  User $i: $j/$TASKS_PER_USER tasks created"
        fi
    done
    
    echo "✓ User $i: All $TASKS_PER_USER tasks created"
done

echo ""
echo "========================================"
echo "✓ Data seeding complete!"
echo "Created: $NUM_USERS users"
echo "Created: $((NUM_USERS * TASKS_PER_USER)) tasks"
echo "========================================"
echo ""
echo "Save this token for benchmarks:"
echo "export AUTH_TOKEN=${TOKENS[1]}"
echo ""


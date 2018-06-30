#!/usr/bin/env zsh

q() {
    if [ "${2:0:1}" != - ] && [ "$2" ];then
        printf "\e[1m%s\e[m\n" "$1"
        # time sqlite3 1.db "$2" > /dev/null
        # sqlite3 1.db "$2" | wc -l
        sqlite3 1.db "$2"
        sqlite3 1.db "EXPLAIN QUERY PLAN $2"
    fi
}

# q 'users per day' '
#       SELECT day, COUNT(DISTINCT user_id), SUM(count)
#         FROM messages
#        WHERE chat_id = -1001281121718
#        GROUP BY day
# '
# 
# q "popular hours" '
#     SELECT hour, sum(count)
#       FROM messages
#      WHERE chat_id = -1001318941241
#      GROUP BY hour
# '
# 
# q "popular weekdays" '
#     SELECT (day + 4)%7, sum(count)
#       FROM messages
#      WHERE chat_id = -1001318941241
#      GROUP BY (day+4) % 7
# '
#
# q 'select piechart' '
#     SELECT messages.user_id, users.full_name, SUM(count)
#       FROM messages
#      INNER JOIN users ON users.user_id = messages.user_id
#      WHERE chat_id = -1001281121718
#      GROUP BY(messages.user_id)
#      ORDER BY SUM(COUNT) DESC
# '

# q 'replies square' '
#     SELECT *
#       FROM replies
# '

q ''

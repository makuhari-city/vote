
curl -d '["fptp", "http://localhost:8101"]' -H "Content-Type:application/json" http://localhost:8100/module/

curl -d '["liquid", "http://localhost:8102"]' -H "Content-Type:application/json" http://localhost:8100/module/

curl -d '["borda", "http://localhost:8103"]' -H "Content-Type:application/json" http://localhost:8100/module/

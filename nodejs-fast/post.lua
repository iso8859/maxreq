wrk.method = "POST"
wrk.body   = [[
{
  "UserName": "user100@example.com",
  "HashedPasswored": "b3351ed9be23d5ad99cc73bdc1aed73913503f064534ead302d7485b72b072fe"
}
]]

wrk.headers["Content-Type"] = "application/json"

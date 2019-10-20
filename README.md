# sigv4 [![actions](https://github.com/softprops/sigv4/workflows/Main/badge.svg)](https://github.com/softprops/sigv4/actions)

> An AWS SigV4 service cli

## usage

todo

## typical setup

Let's say you're a company with a serverless strategy. You expose some private AWS Lambdas behind API GateWay and would like to limit their access to your organizations internal use. The following describes how you might go about doing that.

First you'll need to identify your AWS organiztaion id. You can get this easily from the command line with the `aws` cli.

```sh
$ aws organizations \
	describe-organization \
	--query 'Organization.Id' \
	--output text
```

Secondly you'll need to configure your API GateWay to only allow access to that organization.

With Serverless Framework, you can simply declare a `resourcePolicy` that limits access to your AWS Organization Id and declare an `aws_iam` authorizer for your private functions

```diff
service: sigv4-test

provider:
  name: aws
  runtime: YOUR_DEFAULT_FUNCTION_RUNTIME
+  # https://serverless.com/framework/docs/providers/aws/events/apigateway+/#http-endpoints-with-aws_iam-authorizers
+  # 1) declare resource policy to limit access to your API GateWay
+  resourcePolicy:
+    - Effect: Allow
+      Principal: '*'
+      Action: execute-api:Invoke
+      Resource: arn:aws:execute-api:*
+      Condition:
+        StringEquals:
+          aws:PrincipalOrgID: YOUR_AWS_ORG_ID
+    - Effect: Deny
+      Principal: '*'
+      Action: execute-api:Invoke
+      Resource: arn:aws:execute-api:*
+      Condition:
+        StringNotEquals:
+          aws:PrincipalOrgID: YOUR_AWS_ORG_ID

functions:
  hello:
    handler: YOUR_FUNCTION_HANDLER
    events:
      - http:
          path: '/'
          method: GET
+          # have AWS IAM manage authorizing access to this function
+          # https://serverless.com/framework/docs/providers/aws/events/apigateway/+#http-endpoints-with-aws_iam-authorizers
+          authorizer: aws_iam
```

Doug Tangren (softprops) 2019
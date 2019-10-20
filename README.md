# sigv4 [![actions](https://github.com/softprops/sigv4/workflows/Main/badge.svg)](https://github.com/softprops/sigv4/actions)

> An AWS SigV4 service cli

## usage

todo

## 🗺️ lambda setup

Let's say you're a company with a serverless strategy. You'll likely want to expose some private AWS Lambdas behind API GateWay and would like to limit their access to your organizations internal use. The following describes how you might go about doing that.

First you'll need to identify your **AWS organiztaion id**. You can get this easily from the command line with the `aws` cli.

```sh
$ aws organizations \
	describe-organization \
	--query 'Organization.Id' \
	--output text
```

Secondly you'll need to configure your API GateWay to only allow access to that organization.

With Serverless Framework, you can simply [declare a `resourcePolicy`](https://serverless.com/framework/docs/providers/aws/events/apigateway+/#http-endpoints-with-aws_iam-authorizers) that limits access to your AWS Organization Id and [declare an `aws_iam` authorizer](https://serverless.com/framework/docs/providers/aws/events/apigateway/+#http-endpoints-with-aws_iam-authorizers) for your private functions in your serverless.yml file.

```diff
service: sigv4-test

provider:
  name: aws
  runtime: YOUR_DEFAULT_FUNCTION_RUNTIME
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
+          authorizer: aws_iam
```

## 📝 about sigv4

Security is a first class concern of any modern application. When you offload your services onto managed AWS infrustrcture this is not different when you expose that infrastructure over the internet. AWS offers a built-in security system for managing identity between services called [IAM](https://aws.amazon.com/iam/) and defines a protocol authenticating requests between services that leverages that IAM information called [signature v4 signed requests](https://docs.aws.amazon.com/general/latest/gr/signature-version-4.html).

Doug Tangren (softprops) 2019
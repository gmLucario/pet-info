# Introduction

Lambda function will be triggered from an step function to send reminders to users
TODO: at failure enqueue into a sqs

The previous step function step will send `$.reminder` payload

```json
{
    "when": "datetime-utc",
    "reminder": {
        "phone": "whats-phone-number",
        "body": "Desparasitar galleta"
    }
}
```


## Building

To build the project for production, run `cargo lambda build --release`. Remove the `--release` flag to build for development.

```bash
cargo lambda build --release --arm64 --output-format zip
```

### Using docker image
inside the docker folder:

```bash
docker build -t build_lambda:latest -f lambda_build.Dockerfile --output=out .
```

## Testing

If you want to run integration tests locally, you can use the `cargo lambda watch` and `cargo lambda invoke` commands to do it.

For generic events, where you define the event data structure, you can create a JSON file with the data you want to test with. For example:

```json
{
    "phone": "whats-phone-number",
    "body": "Desparasitar galleta"
}
```

Then, run `cargo lambda invoke --data-file ./data.json` to invoke the function with the data in `data.json`.


# Separate Provider Identity from Provider Protocol

Magnus separates Provider identity from Provider Protocol. A Provider is the configured source of models, while the Provider Protocol is the API format Magnus uses to communicate with it. This lets a Custom Provider represent an external proxy or self-hosted gateway that uses the Anthropic, OpenAI, or Google protocol without treating that provider as the built-in Anthropic, OpenAI, or Google provider.

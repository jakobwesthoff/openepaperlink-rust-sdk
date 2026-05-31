# Add upper bound to get_tags() pagination

`get_tags()` loops until `continuation` is `None`, with only a `next > pos`
guard against backward pagination. A misbehaving AP returning ever-increasing
`continu` values would loop indefinitely. Consider adding a max-iterations
safeguard or a configurable `max_tags` limit.

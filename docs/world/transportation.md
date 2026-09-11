# Transportation

Region connections may specify transportation requirements.

Current known example:

```text
Riptide → Ship
```

The architecture should support future methods such as:

```text
walking
ship
train
carriage
airship
portal
mount
special vehicle
```

Transportation may have:

```text
cost
availability
departure location
campaign requirements
travel events
risk
```

Do not hardcode sea travel as a special one-off Riptide function.

Implement transportation generically enough to support future routes.

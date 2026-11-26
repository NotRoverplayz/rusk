# Layouts

## Rows and columns

```rsml
<row gap="20px" align="center" justify="between">...</row>
<column gap="20px" align="center">...</column>
<center>...</center>
```

## Grid

```rsml
<grid columns="3" tablet-columns="2" mobile-columns="1" gap="20px">
    ...
</grid>
```

Desktop uses `columns`. The generated CSS uses 768px-1024px for tablet and 767px or less for mobile.

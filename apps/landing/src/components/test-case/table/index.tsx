import { Box } from '@devup-ui/react'
import { ComponentProps } from 'react'

export function Table(props: ComponentProps<typeof Box<'table'>>) {
  return (
    <Box
      as="table"
      borderSpacing="0"
      flexGrow="0"
      maxW="100%"
      overflow="hidden"
      w={['100%', null, null, 'fit-content']}
      {...props}
    />
  )
}

export function Thead(props: ComponentProps<typeof Box<'thead'>>) {
  return (
    <Box
      as="thead"
      bg="$tableHeaderBg"
      borderRight="solid 1px $tableHeaderBorder"
      justifyContent="center"
      px="20px"
      py="8px"
      whiteSpace="nowrap"
      {...props}
    />
  )
}

export function Tbody(props: ComponentProps<typeof Box<'tbody'>>) {
  return <Box as="tbody" {...props} />
}

export function Tr(props: ComponentProps<typeof Box<'tr'>>) {
  return <Box as="tr" {...props} />
}

export function Th(props: ComponentProps<typeof Box<'th'>>) {
  return (
    <Box
      as="th"
      bg="$tableHeaderBg"
      borderBottom="solid 1px $tableBorder"
      borderRight="solid 1px $tableHeaderBorder"
      borderTop="solid 1px $tableBorder"
      color="$base"
      justifyContent="center"
      px="20px"
      py="8px"
      selectors={{
        '&:last-child': {
          borderRight: 'solid 1px $tableBorder',
          borderTopRightRadius: '10px',
        },
        '&:first-child': {
          borderTopLeftRadius: '10px',
        },
      }}
      textAlign="left"
      typography="bodyBold"
      {...props}
    />
  )
}

export function Td({
  typography = 'body',
  ...props
}: ComponentProps<typeof Box<'td'>>) {
  return (
    <Box
      as="td"
      borderBottom="solid 1px $tableBorder"
      borderRight="solid 1px $tableBorder"
      justifyContent="center"
      px={[null, null, null, '20px']}
      py="8px"
      selectors={{
        '&:first-child': {
          borderLeft: 'solid 1px $tableBorder',
        },
        'tr[data-responsive="desktop"]:first-of-type &, tr[data-responsive="mobile"]:nth-of-type(2) &':
          {
            borderTop: 'solid 1px $tableBorder',
          },
      }}
      styleOrder={1}
      typography={typography}
      wordBreak="break-all"
      {...props}
    />
  )
}

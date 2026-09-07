import { Box } from '@devup-ui/react'
import { ComponentProps } from 'react'

import { MIDDLE_KOREAN_FONT_FAMILY } from '@/constants/font'

export function Table(props: Omit<ComponentProps<typeof Box<'table'>>, 'as'>) {
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

export function Thead(props: Omit<ComponentProps<typeof Box<'thead'>>, 'as'>) {
  return (
    <Box
      as="thead"
      bg="#2B2B2B"
      borderRight="solid 1px #EFEEEB"
      justifyContent="center"
      px="20px"
      py="8px"
      whiteSpace="nowrap"
      {...props}
    />
  )
}

export function Tbody(props: Omit<ComponentProps<typeof Box<'tbody'>>, 'as'>) {
  return <Box as="tbody" {...props} />
}

export function Tr(props: Omit<ComponentProps<typeof Box<'tr'>>, 'as'>) {
  return <Box as="tr" {...props} />
}

export function Th(props: Omit<ComponentProps<typeof Box<'th'>>, 'as'>) {
  return (
    <Box
      as="th"
      bg="$primary"
      borderBottom="solid 1px $primary"
      borderRight="solid 1px $background"
      borderTop="solid 1px $primary"
      color="$base"
      justifyContent="center"
      px="20px"
      py="8px"
      selectors={{
        '&:last-child': {
          borderRight: 'solid 1px $primary',
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
}: Omit<ComponentProps<typeof Box<'td'>>, 'as'>) {
  return (
    <Box
      as="td"
      borderBottom="solid 1px $primary"
      borderRight="solid 1px $primary"
      fontFamily={MIDDLE_KOREAN_FONT_FAMILY}
      justifyContent="center"
      px={[null, null, null, '20px']}
      py="8px"
      selectors={{
        '&:first-child': {
          borderLeft: 'solid 1px $primary',
        },
        'tr[data-responsive="desktop"]:first-of-type &, tr[data-responsive="mobile"]:nth-of-type(2) &':
          {
            borderTop: 'solid 1px $primary',
          },
      }}
      styleOrder={1}
      typography={typography}
      wordBreak="break-all"
      {...props}
    />
  )
}

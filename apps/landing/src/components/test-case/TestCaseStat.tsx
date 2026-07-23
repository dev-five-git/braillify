import { Box, Center, css, Flex, Text } from '@devup-ui/react'
import { ComponentProps } from 'react'

import Tooltip from './Tooltip'

interface CompetitorStatProps {
  label: string
  total: number
  fail: number
}

function CompetitorStat({ label, total, fail }: CompetitorStatProps) {
  const success = total - fail
  const percent = Math.round((success / total) * 100)

  return (
    <Flex alignItems="center" gap="10px" whiteSpace="nowrap">
      <Text color="#FFF" typography="progress">
        {label}
      </Text>
      <Text color="#FFF" typography="body" wordBreak="keep-all">
        성공 {(total - fail).toLocaleString()}
      </Text>
      <Text color="$error" typography="body" wordBreak="keep-all">
        실패 {fail.toLocaleString()}
      </Text>
      <Text color="#FFF" typography="body">
        ({percent}%)
      </Text>
    </Flex>
  )
}

interface TestCaseStatProps extends ComponentProps<typeof Center<'div'>> {
  showTotal?: boolean
  colorPercentage?: boolean
  total: number
  success: number
  fail: number
  worldTotal?: number
  worldFail?: number
  jeomsarangTotal?: number
  jeomsarangFail?: number
}

export function TestCaseStat({
  showTotal = false,
  colorPercentage = true,
  total,
  success,
  fail,
  worldTotal,
  worldFail,
  jeomsarangTotal,
  jeomsarangFail,
  ...props
}: TestCaseStatProps) {
  const hasFail = fail > 0
  const braillifyPercent = Math.round((success / total) * 100)

  const hasWorld = worldTotal != null && worldTotal > 0
  const hasJeomsarang = jeomsarangTotal != null && jeomsarangTotal > 0

  return (
    <Flex
      alignItems="center"
      flexWrap="wrap"
      gap={['4px', null, null, '8px']}
      styleOrder={1}
    >
      <Center
        bg="$menuHover"
        borderRadius="10px"
        gap="10px"
        px="16px"
        py="10px"
        {...props}
      >
        {showTotal && (
          <Text color="$text" typography="progress">
            전체 {total.toLocaleString()}
          </Text>
        )}
        <Text color="$text" typography="progress">
          성공 {success.toLocaleString()}
        </Text>
        <Text color={hasFail ? '$error' : '$text'} typography="progress">
          실패 {fail.toLocaleString()}
        </Text>
        <Text
          color={!colorPercentage ? '$text' : hasFail ? '$error' : '$success'}
          typography="progress"
        >
          ({braillifyPercent}%)
        </Text>
      </Center>
      <Center
        bg="$menuHover"
        borderRadius="10px"
        data-group
        p="10px"
        pos="relative"
      >
        <Box
          aspectRatio="1"
          bg="$text"
          boxSize="20px"
          maskImage="url('/images/plus.svg')"
          maskPos="center"
          maskRepeat="no-repeat"
          maskSize="contain"
        />
        <Tooltip
          className={css({
            gap: '8px',
            py: '12px',
            px: '16px',
            bottom: '0',
            left: '0',
            borderRadius: '10px',
          })}
          translateY="calc(100% + 4px)"
        >
          <CompetitorStat fail={fail} label="Braillfy" total={total} />
          {hasWorld && (
            <CompetitorStat
              fail={worldFail!}
              label="점자세상"
              total={worldTotal}
            />
          )}
          {hasJeomsarang && (
            <CompetitorStat
              fail={jeomsarangFail!}
              label="점사랑"
              total={jeomsarangTotal}
            />
          )}
        </Tooltip>
      </Center>
    </Flex>
  )
}

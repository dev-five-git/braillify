import { type DotNumber } from '@braillify/shared'
import { Box, Flex, Text } from '@devup-ui/react'

type Props = {
  mask: number
  index: number
  onToggleDot: (dot: DotNumber) => void
  onRemove?: () => void
}

const DOT_LAYOUT: Array<{ dot: DotNumber }> = [
  { dot: 1 },
  { dot: 2 },
  { dot: 3 },
  { dot: 4 },
  { dot: 5 },
  { dot: 6 },
]

function dotBit(dot: DotNumber): number {
  return 1 << (dot - 1)
}

export function EditableBrailleCell({
  mask,
  index,
  onToggleDot,
  onRemove,
}: Props) {
  return (
    <Flex alignItems="center" flexDirection="column" gap="6px">
      <Text color="$textSubtle" fontSize="11px">
        #{index + 1}
      </Text>
      <Box bg="$bg" borderRadius="12px" px="14px" py="12px">
        <Box
          display="grid"
          gap="6px"
          gridAutoFlow="column"
          gridTemplateColumns="repeat(2, 22px)"
          gridTemplateRows="repeat(3, 22px)"
        >
          {DOT_LAYOUT.map(({ dot }) => {
            const active = (mask & dotBit(dot)) !== 0
            return (
              <Box
                key={dot}
                aria-label={`${index + 1}번 셀 점 ${dot}`}
                aria-pressed={active}
                as="button"
                bg={active ? '$primary' : '$surface'}
                border="1.5px solid"
                borderColor={active ? '$primary' : '$border'}
                borderRadius="50%"
                h="22px"
                onClick={() => onToggleDot(dot)}
                p={0}
                type="button"
                w="22px"
              />
            )
          })}
        </Box>
      </Box>
      {onRemove && (
        <Box
          aria-label="셀 삭제"
          as="button"
          bg="transparent"
          border={0}
          color="$textSubtle"
          fontSize="18px"
          lineHeight={1}
          onClick={onRemove}
          p={0}
          type="button"
        >
          ×
        </Box>
      )}
    </Flex>
  )
}

import { Box, Flex } from '@devup-ui/react'
import { useState } from 'react'

import { BottomTabBar, type TabKey } from '../components/BottomTabBar'
import { EditorView } from '../views/EditorView'
import { HistoryView } from '../views/HistoryView'
import { TranslatorView } from '../views/TranslatorView'

function App() {
  const [tab, setTab] = useState<TabKey>('translator')

  return (
    <Flex
      bg="$bg"
      flexDirection="column"
      height="100vh"
      overflow="hidden"
      width="100%"
    >
      <Box as="main" flex={1} overflowY="auto" pb="8px">
        {tab === 'translator' && <TranslatorView />}
        {tab === 'editor' && <EditorView />}
        {tab === 'history' && <HistoryView />}
      </Box>
      <BottomTabBar active={tab} onChange={setTab} />
    </Flex>
  )
}

export default App

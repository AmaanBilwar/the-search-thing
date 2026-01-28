import { handle } from '@/lib/main/shared'
import axios from 'axios';

export const registerSearchHandlers = () => {
  handle('search', async (query: string) => {
    const response = await axios.post('http://localhost:3000/api/search', {
      params: { q: query }
    });
    const results = response.data.results;
    return { results };
  });
  
  handle('check', async (query: string) => {
    const response = await axios.post('http://localhost:3000/api/search', {
      params: { q: query }
    });
    const results = response.data.results;
    return { results };
  })
  
  handle('index', async (query: string) => {
    const response = await axios.post('http://localhost:3000/api/search', {
      params: { q: query }
    });
    const results = response.data.results;
    return { results };
  })
}

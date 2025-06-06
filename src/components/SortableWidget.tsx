import React from 'react';
import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { GripVertical } from 'lucide-react';

interface SortableWidgetProps {
  id: string;
  children: React.ReactNode;
  isDraggable?: boolean;
}

export const SortableWidget: React.FC<SortableWidgetProps> = ({ id, children, isDraggable = false }) => {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div ref={setNodeRef} style={style} className="relative group">
      {isDraggable && (
        <div
          {...attributes}
          {...listeners}
          className="absolute -top-2 -left-2 p-2 bg-gray-200 dark:bg-gray-700 rounded-lg opacity-0 group-hover:opacity-100 transition-opacity cursor-move z-10"
        >
          <GripVertical className="w-4 h-4 text-gray-600 dark:text-gray-400" />
        </div>
      )}
      {children}
    </div>
  );
};
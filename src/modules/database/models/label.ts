import type { Insertable } from 'kysely'
import { sql } from 'kysely'
import type { Label as TransformLabel } from '../transform-types'
import { Model, get, set } from './model'

export class Label extends Model<TransformLabel> {
  table = 'label' as const

  transactionInsert(@set value: Insertable<TransformLabel>) {
    return this.transaction().execute(async (trx) => {
      const { lastInsertId } = await trx.label.insert(value)
      await trx.label.update(lastInsertId, {
        sort: lastInsertId,
      })
    })
  }

  removeRelation(id: number) {
    return this.transaction().execute(async (trx) => {
      await trx.label.remove(id)
      await trx.note.removeBy({
        labelId: id,
      })
    })
  }

  batchRemoveRelation(idList: number[]) {
    return this.transaction().execute(trx => Promise.all(idList.map(id => trx.label.removeRelation(id))))
  }

  removeBy(value: { planId?: number }) {
    let query = this.baseRemove()
    if (value.planId)
      query = query.where('planId', '=', value.planId)

    return query
  }

  @get()
  select(value?: { id?: number }) {
    const query = this.selectByLooseType(value)
    return query
      .select([
        'label.id',
        'label.name',
        'label.color',
        'label.sort',
        'label.planId',
        'label.deletedAt',
        'label.createdAt',
        'label.updatedAt',
        sql<number>`ifnull(sum(n.end - n.start), 0)`.as('totalTime'),
      ])
      .leftJoin('note as n', join => join.onRef('n.labelId', '=', 'label.id').on('n.deletedAt', '=', 0))
      .groupBy('label.id')
      .orderBy(['label.sort'])
  }
}
